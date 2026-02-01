//! Sandbox traits and types

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Configuration for sandbox execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    /// Docker image to use (e.g., "python:3.11-slim", "node:20-slim")
    pub image: Option<String>,
    /// Memory limit (e.g., "256m", "1g")
    pub memory_limit: Option<String>,
    /// CPU limit (e.g., 0.5, 1.0, 2.0)
    pub cpu_limit: Option<f32>,
    /// Timeout in seconds
    pub timeout_seconds: Option<u32>,
    /// Whether network access is enabled
    pub network_enabled: Option<bool>,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            image: Some("debian:stable-slim".to_string()),
            memory_limit: Some("256m".to_string()),
            cpu_limit: Some(0.5),
            timeout_seconds: Some(300),
            network_enabled: Some(false),
        }
    }
}

impl SandboxConfig {
    /// Create a new sandbox config with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the Docker image
    pub fn with_image(mut self, image: impl Into<String>) -> Self {
        self.image = Some(image.into());
        self
    }

    /// Set the memory limit
    pub fn with_memory_limit(mut self, limit: impl Into<String>) -> Self {
        self.memory_limit = Some(limit.into());
        self
    }

    /// Set the CPU limit
    pub fn with_cpu_limit(mut self, limit: f32) -> Self {
        self.cpu_limit = Some(limit);
        self
    }

    /// Set the timeout
    pub fn with_timeout(mut self, seconds: u32) -> Self {
        self.timeout_seconds = Some(seconds);
        self
    }

    /// Enable or disable network access
    pub fn with_network(mut self, enabled: bool) -> Self {
        self.network_enabled = Some(enabled);
        self
    }

    /// Create a Python-specific config
    pub fn python() -> Self {
        Self::default().with_image("python:3.11-slim")
    }

    /// Create a Node.js-specific config
    pub fn nodejs() -> Self {
        Self::default().with_image("node:20-slim")
    }
}

/// Result of sandbox execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Whether the command executed successfully
    pub success: bool,
    /// Standard output
    pub stdout: String,
    /// Standard error
    pub stderr: String,
    /// Exit code
    pub exit_code: i32,
    /// Whether the command timed out
    pub timed_out: bool,
}

impl ExecutionResult {
    /// Create a successful result
    pub fn success(stdout: String) -> Self {
        Self {
            success: true,
            stdout,
            stderr: String::new(),
            exit_code: 0,
            timed_out: false,
        }
    }

    /// Create a failed result
    pub fn failure(stderr: String, exit_code: i32) -> Self {
        Self {
            success: false,
            stdout: String::new(),
            stderr,
            exit_code,
            timed_out: false,
        }
    }

    /// Create a timeout result
    pub fn timeout() -> Self {
        Self {
            success: false,
            stdout: String::new(),
            stderr: "Command timed out".to_string(),
            exit_code: 124,
            timed_out: true,
        }
    }
}

/// Trait for sandbox implementations
#[async_trait]
pub trait Sandbox: Send + Sync {
    /// Get the name of this sandbox implementation
    fn name(&self) -> &str;

    /// Check if this sandbox is available
    async fn is_available(&self) -> bool;

    /// Execute a command in the sandbox
    ///
    /// # Arguments
    /// * `command` - The command to execute
    /// * `workspace` - Path to mount as the working directory
    /// * `config` - Sandbox configuration
    async fn execute(
        &self,
        command: &str,
        workspace: &Path,
        config: &SandboxConfig,
    ) -> Result<ExecutionResult, String>;

    /// Execute a command with additional files mounted
    ///
    /// # Arguments
    /// * `command` - The command to execute
    /// * `workspace` - Path to mount as the working directory
    /// * `extra_files` - Optional path to mount as read-only files
    /// * `config` - Sandbox configuration
    async fn execute_with_files(
        &self,
        command: &str,
        workspace: &Path,
        extra_files: Option<&Path>,
        config: &SandboxConfig,
    ) -> Result<ExecutionResult, String> {
        // Default implementation ignores extra files
        let _ = extra_files;
        self.execute(command, workspace, config).await
    }
}
