//! Agent configuration

use crate::config::ExecutionMode;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Agent ID
    pub id: String,
    /// Agent name
    pub name: String,
    /// AI provider (openai, anthropic, gemini)
    pub provider: String,
    /// AI model name
    pub model: String,
    /// System prompt/preamble
    pub system_prompt: Option<String>,
    /// Maximum number of turns for multi-turn execution
    pub max_turns: usize,
    /// Workspace path for file operations
    pub workspace_path: PathBuf,
    /// Execution mode (sandbox, direct, flexible)
    pub execution_mode: ExecutionMode,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Default Agent".to_string(),
            provider: "openai".to_string(),
            model: "gpt-4o".to_string(),
            system_prompt: None,
            max_turns: 10,
            workspace_path: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            execution_mode: ExecutionMode::Flexible,
        }
    }
}

impl AgentConfig {
    /// Create a new agent config
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            ..Default::default()
        }
    }

    /// Set the AI provider
    pub fn with_provider(mut self, provider: impl Into<String>) -> Self {
        self.provider = provider.into();
        self
    }

    /// Set the AI model
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Set the system prompt
    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }

    /// Set the maximum number of turns
    pub fn with_max_turns(mut self, max_turns: usize) -> Self {
        self.max_turns = max_turns;
        self
    }

    /// Set the workspace path
    pub fn with_workspace(mut self, path: PathBuf) -> Self {
        self.workspace_path = path;
        self
    }

    /// Set the execution mode
    pub fn with_execution_mode(mut self, mode: ExecutionMode) -> Self {
        self.execution_mode = mode;
        self
    }
}
