//! Skill types

use serde::{Deserialize, Serialize};

/// Parsed skill from SKILL.md
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedSkill {
    /// Skill name (hyphen-case, max 64 chars)
    pub name: String,
    /// Skill description
    pub description: String,
    /// License
    pub license: Option<String>,
    /// Category
    pub category: Option<String>,
    /// Trigger commands
    pub triggers: Option<Vec<String>>,
    /// Whether sandbox is required
    pub requires_sandbox: bool,
    /// Sandbox configuration
    pub sandbox_config: Option<SkillSandboxConfig>,
    /// Execution mode ("sandbox", "direct", "flexible")
    pub execution_mode: Option<String>,
    /// Body content (markdown)
    pub body: String,
}

/// Sandbox configuration for skill execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillSandboxConfig {
    /// Docker image
    pub image: Option<String>,
    /// Memory limit (e.g., "256m")
    pub memory_limit: Option<String>,
    /// CPU limit
    pub cpu_limit: Option<f32>,
    /// Timeout in seconds
    pub timeout_seconds: Option<u32>,
    /// Whether network is enabled
    pub network_enabled: Option<bool>,
}

impl Default for SkillSandboxConfig {
    fn default() -> Self {
        Self {
            image: Some("python:3.11-slim".to_string()),
            memory_limit: Some("256m".to_string()),
            cpu_limit: Some(0.5),
            timeout_seconds: Some(300),
            network_enabled: Some(false),
        }
    }
}

impl From<SkillSandboxConfig> for crate::sandbox::SandboxConfig {
    fn from(skill_config: SkillSandboxConfig) -> Self {
        crate::sandbox::SandboxConfig {
            image: skill_config.image,
            memory_limit: skill_config.memory_limit,
            cpu_limit: skill_config.cpu_limit,
            timeout_seconds: skill_config.timeout_seconds,
            network_enabled: skill_config.network_enabled,
        }
    }
}
