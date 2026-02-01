//! Core configuration types

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Core configuration for AnyCowork
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreConfig {
    /// Default workspace path for file operations
    pub workspace_path: PathBuf,

    /// Default AI provider (openai, anthropic, gemini)
    pub default_provider: String,

    /// Default AI model
    pub default_model: String,

    /// Maximum number of agent turns per request
    pub max_agent_turns: usize,

    /// Default execution mode (sandbox, direct, flexible)
    pub execution_mode: ExecutionMode,

    /// Whether to enable debug logging
    pub debug: bool,
}

impl Default for CoreConfig {
    fn default() -> Self {
        Self {
            workspace_path: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            default_provider: "openai".to_string(),
            default_model: "gpt-4o".to_string(),
            max_agent_turns: 10,
            execution_mode: ExecutionMode::Flexible,
            debug: false,
        }
    }
}

/// Execution mode for tools and skills
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionMode {
    /// Always use sandboxed execution (Docker)
    Sandbox,
    /// Always use direct execution (native)
    Direct,
    /// Use sandbox if available, otherwise direct
    Flexible,
}

impl Default for ExecutionMode {
    fn default() -> Self {
        Self::Flexible
    }
}

impl std::fmt::Display for ExecutionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionMode::Sandbox => write!(f, "sandbox"),
            ExecutionMode::Direct => write!(f, "direct"),
            ExecutionMode::Flexible => write!(f, "flexible"),
        }
    }
}

impl std::str::FromStr for ExecutionMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "sandbox" => Ok(ExecutionMode::Sandbox),
            "direct" => Ok(ExecutionMode::Direct),
            "flexible" => Ok(ExecutionMode::Flexible),
            _ => Err(format!("Unknown execution mode: {}", s)),
        }
    }
}
