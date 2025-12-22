//! Installer Module
//!
//! Trait-based adapter pattern for installing agents to different editors.

mod claude;
mod cursor;
mod codex;

use anyhow::Result;

pub use claude::ClaudeInstaller;
pub use cursor::CursorInstaller;
pub use codex::CodexInstaller;

use crate::core::agent::AgentConfig;

/// Target editor for installation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Claude,
    Cursor,
    Codex,
}

impl Target {
    /// Get the display name for the target
    pub fn display_name(&self) -> &'static str {
        match self {
            Target::Claude => "Claude Code",
            Target::Cursor => "Cursor",
            Target::Codex => "Codex",
        }
    }
}

/// Installer trait - the adapter pattern for different editors
pub trait Installer: Send + Sync {
    /// Install the agent's identity (system prompt)
    fn install_identity(&self, agent: &AgentConfig) -> Result<()>;

    /// Install the agent's skills (knowledge base)
    fn install_skills(&self, agent: &AgentConfig) -> Result<()>;

    /// Install the agent's MCP tools
    fn install_tools(&self, agent: &AgentConfig) -> Result<()>;

    /// Uninstall an agent by name
    fn uninstall(&self, agent_name: &str) -> Result<()>;
}

/// Get the appropriate installer for a target
pub fn get_installer(target: Target, global: bool) -> Box<dyn Installer> {
    match target {
        Target::Claude => Box::new(ClaudeInstaller::new(global)),
        Target::Cursor => Box::new(CursorInstaller::new(global)),
        Target::Codex => Box::new(CodexInstaller::new(global)),
    }
}
