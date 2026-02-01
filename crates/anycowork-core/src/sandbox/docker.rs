//! Docker sandbox implementation

use super::traits::{ExecutionResult, Sandbox, SandboxConfig};
use async_trait::async_trait;
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;

/// Predefined Docker images for skill execution
#[derive(Debug, Clone)]
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

/// Docker sandbox for secure command execution
pub struct DockerSandbox {
    docker_available: bool,
}

impl DockerSandbox {
    /// Create a new Docker sandbox
    pub async fn new() -> Result<Self, String> {
        let available = Self::check_available().await;
        Ok(Self {
            docker_available: available,
        })
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

    /// Execute a Python script in sandbox
    pub async fn execute_python(
        &self,
        script_path: &str,
        workspace: &Path,
        skill_files: Option<&Path>,
        config: &SandboxConfig,
    ) -> Result<ExecutionResult, String> {
        let command = format!("python3 /skill/{}", script_path);
        self.execute_with_files(&command, workspace, skill_files, config)
            .await
    }

    /// Execute a shell script in sandbox
    pub async fn execute_shell(
        &self,
        script_path: &str,
        workspace: &Path,
        skill_files: Option<&Path>,
        config: &SandboxConfig,
    ) -> Result<ExecutionResult, String> {
        let command = format!("sh /skill/{}", script_path);
        self.execute_with_files(&command, workspace, skill_files, config)
            .await
    }

    /// Execute a Node.js script in sandbox
    pub async fn execute_node(
        &self,
        script_path: &str,
        workspace: &Path,
        skill_files: Option<&Path>,
        config: &SandboxConfig,
    ) -> Result<ExecutionResult, String> {
        let mut node_config = config.clone();
        if node_config.image.is_none() {
            node_config.image = Some("node:20-slim".to_string());
        }

        let command = format!("node /skill/{}", script_path);
        self.execute_with_files(&command, workspace, skill_files, &node_config)
            .await
    }
}

#[async_trait]
impl Sandbox for DockerSandbox {
    fn name(&self) -> &str {
        "docker"
    }

    async fn is_available(&self) -> bool {
        self.docker_available
    }

    async fn execute(
        &self,
        command: &str,
        workspace: &Path,
        config: &SandboxConfig,
    ) -> Result<ExecutionResult, String> {
        self.execute_with_files(command, workspace, None, config)
            .await
    }

    async fn execute_with_files(
        &self,
        command: &str,
        workspace: &Path,
        extra_files: Option<&Path>,
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
        let mut args = vec!["run".to_string(), "--rm".to_string()];

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
        let workspace_abs = workspace
            .canonicalize()
            .map_err(|e| format!("Invalid workspace path: {}", e))?;
        args.push("-v".to_string());
        args.push(format!("{}:/workspace:rw", workspace_abs.display()));

        // Mount extra files if provided (read-only)
        if let Some(extra_path) = extra_files {
            let extra_abs = extra_path
                .canonicalize()
                .map_err(|e| format!("Invalid extra files path: {}", e))?;
            args.push("-v".to_string());
            args.push(format!("{}:/skill:ro", extra_abs.display()));
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
}
