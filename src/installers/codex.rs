//! Codex Installer
//!
//! Installs agent configurations into Codex's native format.
//!
//! Output structure:
//! - ~/.codex/agents/{name}.md - Agent as Markdown with YAML frontmatter
//! - ~/.codex/skills/{name}/{skill}.md - Skills as Markdown files
//! - ~/.codex/config.json - MCP tool configuration (assumed)

use anyhow::{Context, Result};
use serde_json::{json, Value};
use colored::Colorize;
use std::fs;
use std::path::PathBuf;

use super::Installer;
use crate::core::agent::AgentConfig;
use crate::utils::paths;

/// Installer for Codex
pub struct CodexInstaller {
    /// Whether to install globally
    global: bool,
}

impl CodexInstaller {
    pub fn new(global: bool) -> Self {
        Self { global }
    }

    /// Get the base directory for Codex configuration
    fn get_base_dir(&self) -> Result<PathBuf> {
        paths::codex_config_dir()
            .context("Could not find Codex configuration directory")
    }

    /// Get the agents directory
    fn get_agents_dir(&self) -> Result<PathBuf> {
        Ok(self.get_base_dir()?.join("agents"))
    }

    /// Get the Codex config path (assumed)
    fn get_config_path(&self) -> Result<PathBuf> {
        Ok(self.get_base_dir()?.join("config.json"))
    }

    /// Generate the markdown content with YAML frontmatter
    fn generate_agent_markdown(agent: &AgentConfig) -> String {
        let icon = agent.identity.icon.as_deref().unwrap_or("🤖");
        let model = agent.identity.model.as_deref().unwrap_or("gpt-4o");
        
        format!(
            r#"---
name: {}
description: {}
model: {}
icon: {}
---

{}"#,
            agent.name,
            agent.description,
            model,
            icon,
            agent.identity.system_prompt
        )
    }
}

impl Installer for CodexInstaller {
    fn install_identity(&self, agent: &AgentConfig) -> Result<()> {
        let agents_dir = self.get_agents_dir()?;
        fs::create_dir_all(&agents_dir)?;

        // Create the agent markdown file
        let agent_file = agents_dir.join(format!("{}.md", agent.name));
        let markdown_content = Self::generate_agent_markdown(agent);
        
        fs::write(&agent_file, markdown_content)?;

        Ok(())
    }

    fn install_skills(&self, agent: &AgentConfig) -> Result<()> {
        if agent.skills.is_empty() {
            return Ok(());
        }

        let base_dir = self.get_base_dir()?;
        let skills_dir = base_dir.join("skills").join(&agent.name);
        fs::create_dir_all(&skills_dir)?;

        for skill in &agent.skills {
            let skill_file = skills_dir.join(format!("{}.md", skill.name));
            fs::write(&skill_file, &skill.content)?;
        }

        Ok(())
    }

    fn install_tools(&self, agent: &AgentConfig) -> Result<()> {
        if agent.mcp.is_empty() {
            return Ok(());
        }

        let config_path = self.get_config_path()?;

        // Load existing config or create new one
        let mut config: Value = if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            serde_json::from_str(&content).unwrap_or_else(|_| json!({}))
        } else {
            json!({})
        };

        // Ensure mcpServers object exists
        if config.get("mcpServers").is_none() {
            config["mcpServers"] = json!({});
        }

        // Add each MCP tool
        for tool in &agent.mcp {
            let tool_config = json!({
                "command": tool.command,
                "args": tool.args,
                "env": tool.env
            });
            config["mcpServers"][&tool.name] = tool_config;

            // Check for setup URL (API key requirement)
            if let Some(url) = &tool.setup_url {
                println!("\n  {} Setup required for MCP tool '{}'", "ℹ".blue().bold(), tool.name.bold());
                println!("  {} Get your API key here: {}", "→".cyan(), url.underline().blue());
            }
        }

        // Ensure parent directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Write the updated config
        fs::write(&config_path, serde_json::to_string_pretty(&config)?)?;

        Ok(())
    }

    fn uninstall(&self, agent_name: &str) -> Result<()> {
        // Remove agent file
        let agent_file = self.get_agents_dir()?.join(format!("{}.md", agent_name));
        if agent_file.exists() {
            fs::remove_file(&agent_file)?;
        }

        // Remove skills directory
        let skills_dir = self.get_base_dir()?.join("skills").join(agent_name);
        if skills_dir.exists() {
            fs::remove_dir_all(&skills_dir)?;
        }

        Ok(())
    }
}
