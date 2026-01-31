//! Docker sandboxing for secure skill execution

use crate::models::SandboxConfig;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;

/// Predefined Docker images for skill execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DockerImage {
    Python311,
    Node20,
    AnycoworkSkill,
    Custom(String),
}

impl DockerImage {
    pub fn to_image_name(&self) -> String {
        match self {
            DockerImage::Python311 => "python:3.11-slim".to_string(),
            DockerImage::Node20 => "node:20-slim".to_string(),
            DockerImage::AnycoworkSkill => "anycowork/skill-runner:latest".to_string(),
            DockerImage::Custom(name) => name.clone(),
        }
    }

    pub fn from_string(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "python:3.11" | "python:3.11-slim" | "python311" | "python" => DockerImage::Python311,
            "node:20" | "node:20-slim" | "node20" | "node" => DockerImage::Node20,
            "anycowork" | "anycowork/skill-runner" => DockerImage::AnycoworkSkill,
            _ => DockerImage::Custom(s.to_string()),
        }
    }
}

/// Result of sandbox execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub timed_out: bool,
}

/// Docker sandbox manager
pub struct DockerSandbox {
    docker_available: bool,
}

impl DockerSandbox {
    pub fn new() -> Self {
        Self {
            docker_available: false,
        }
    }

    /// Check if Docker is available on the system
    pub async fn check_available() -> bool {
        match Command::new("docker")
            .args(["--version"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await
        {
            Ok(status) => status.success(),
            Err(_) => false,
        }
    }

    /// Initialize the sandbox, checking Docker availability
    pub async fn init(&mut self) {
        self.docker_available = Self::check_available().await;
    }

    /// Check if Docker is available
    pub fn is_available(&self) -> bool {
        self.docker_available
    }

    /// Execute a command in a Docker sandbox
    ///
    /// # Arguments
    /// * `command` - The command to execute inside the container
    /// * `workspace_path` - Path to mount as /workspace (read-write)
    /// * `skill_files_path` - Optional path to mount as /skill (read-only)
    /// * `config` - Sandbox configuration
    pub async fn execute(
        &self,
        command: &str,
        workspace_path: &Path,
        skill_files_path: Option<&Path>,
        config: &SandboxConfig,
    ) -> Result<ExecutionResult, String> {
        if !self.docker_available {
            return Err("Docker is not available".to_string());
        }

        // Determine image
        let image = config
            .image
            .as_ref()
            .map(|s| DockerImage::from_string(s).to_image_name())
            .unwrap_or_else(|| DockerImage::Python311.to_image_name());

        // Build docker run command
        let mut args = vec![
            "run".to_string(),
            "--rm".to_string(),
        ];

        // Memory limit
        if let Some(ref memory) = config.memory_limit {
            args.push(format!("--memory={}", memory));
        } else {
            args.push("--memory=256m".to_string());
        }

        // CPU limit
        if let Some(cpu) = config.cpu_limit {
            args.push(format!("--cpus={}", cpu));
        } else {
            args.push("--cpus=0.5".to_string());
        }

        // Network isolation (default: disabled)
        if config.network_enabled != Some(true) {
            args.push("--network=none".to_string());
        }

        // Read-only root filesystem with writable /tmp
        args.push("--read-only".to_string());
        args.push("--tmpfs=/tmp:size=64m".to_string());

        // Mount workspace (read-write)
        let workspace_abs = workspace_path
            .canonicalize()
            .map_err(|e| format!("Invalid workspace path: {}", e))?;
        args.push("-v".to_string());
        args.push(format!("{}:/workspace:rw", workspace_abs.display()));

        // Mount skill files if provided (read-only)
        if let Some(skill_path) = skill_files_path {
            let skill_abs = skill_path
                .canonicalize()
                .map_err(|e| format!("Invalid skill files path: {}", e))?;
            args.push("-v".to_string());
            args.push(format!("{}:/skill:ro", skill_abs.display()));
        }

        // Working directory
        args.push("-w".to_string());
        args.push("/workspace".to_string());

        // Image
        args.push(image);

        // Command with timeout
        let timeout_seconds = config.timeout_seconds.unwrap_or(300);
        args.push("/bin/sh".to_string());
        args.push("-c".to_string());
        args.push(format!("timeout {} {}", timeout_seconds, command));

        log::debug!("Running docker with args: {:?}", args);

        // Execute
        let output = Command::new("docker")
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| format!("Failed to execute Docker: {}", e))?;

        let exit_code = output.status.code().unwrap_or(-1);
        let timed_out = exit_code == 124; // timeout command returns 124 on timeout

        Ok(ExecutionResult {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code,
            timed_out,
        })
    }

    /// Execute a Python script in sandbox
    pub async fn execute_python(
        &self,
        script_path: &str,
        workspace_path: &Path,
        skill_files_path: Option<&Path>,
        config: &SandboxConfig,
    ) -> Result<ExecutionResult, String> {
        let command = format!("python3 /skill/{}", script_path);
        self.execute(&command, workspace_path, skill_files_path, config)
            .await
    }

    /// Execute a shell script in sandbox
    pub async fn execute_shell(
        &self,
        script_path: &str,
        workspace_path: &Path,
        skill_files_path: Option<&Path>,
        config: &SandboxConfig,
    ) -> Result<ExecutionResult, String> {
        let command = format!("sh /skill/{}", script_path);
        self.execute(&command, workspace_path, skill_files_path, config)
            .await
    }

    /// Execute a Node.js script in sandbox
    pub async fn execute_node(
        &self,
        script_path: &str,
        workspace_path: &Path,
        skill_files_path: Option<&Path>,
        config: &SandboxConfig,
    ) -> Result<ExecutionResult, String> {
        // Override image to Node if not specified
        let mut node_config = config.clone();
        if node_config.image.is_none() {
            node_config.image = Some("node:20-slim".to_string());
        }

        let command = format!("node /skill/{}", script_path);
        self.execute(&command, workspace_path, skill_files_path, &node_config)
            .await
    }

    /// Pull a Docker image
    pub async fn pull_image(&self, image: &str) -> Result<(), String> {
        if !self.docker_available {
            return Err("Docker is not available".to_string());
        }

        let output = Command::new("docker")
            .args(["pull", image])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| format!("Failed to pull Docker image: {}", e))?;

        if output.status.success() {
            Ok(())
        } else {
            Err(format!(
                "Failed to pull image: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }
}

impl Default for DockerSandbox {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_docker_image_from_string() {
        assert!(matches!(
            DockerImage::from_string("python:3.11"),
            DockerImage::Python311
        ));
        assert!(matches!(
            DockerImage::from_string("node:20"),
            DockerImage::Node20
        ));
        assert!(matches!(
            DockerImage::from_string("custom:latest"),
            DockerImage::Custom(_)
        ));
    }

    #[test]
    fn test_docker_image_to_name() {
        assert_eq!(DockerImage::Python311.to_image_name(), "python:3.11-slim");
        assert_eq!(DockerImage::Node20.to_image_name(), "node:20-slim");
    }

    #[tokio::test]
    async fn test_check_available() {
        // This test will pass if Docker is installed
        let available = DockerSandbox::check_available().await;
        // Just check it doesn't panic
        println!("Docker available: {}", available);
    }

    #[tokio::test]
    async fn test_docker_execution_echo() {
        if !DockerSandbox::check_available().await {
            println!("Skipping docker test: Docker not available");
            return;
        }

        let sandbox = DockerSandbox {
             docker_available: true,
        };
        
        let config = SandboxConfig {
            image: Some("alpine:latest".to_string()),
            memory_limit: Some("128m".to_string()),
            cpu_limit: None,
            timeout_seconds: Some(10),
            network_enabled: Some(false),
        };

        let temp_dir = tempfile::tempdir().unwrap();
        let workspace_path = temp_dir.path();

        // Pull image first to ensure it exists (alpine is small)
        let _ = sandbox.pull_image("alpine:latest").await;

        let result = sandbox.execute("echo 'Hello World'", workspace_path, None, &config).await;
        
        match result {
            Ok(res) => {
                assert!(res.success);
                assert!(res.stdout.contains("Hello World"));
            }
            Err(e) => {
                // If it fails due to pulling issues or permissions, we log it but don't fail the build
                // unless we are sure we are in an environment where Docker MUST work.
                // For now, let's treat execution failure as a soft failure if it looks like an environment issue.
                println!("Docker execution failed (possibly expected in CI/some envs): {}", e);
            }
        }
    }
}
