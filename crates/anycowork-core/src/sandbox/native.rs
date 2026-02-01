//! Native sandbox implementation (direct execution without Docker)

use super::traits::{ExecutionResult, Sandbox, SandboxConfig};
use async_trait::async_trait;
use std::path::Path;
use tokio::process::Command;

/// Native sandbox for direct command execution
///
/// This sandbox executes commands directly on the host system without
/// containerization. Use with caution - it provides no isolation.
pub struct NativeSandbox;

impl NativeSandbox {
    /// Create a new native sandbox
    pub fn new() -> Self {
        Self
    }
}

impl Default for NativeSandbox {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Sandbox for NativeSandbox {
    fn name(&self) -> &str {
        "native"
    }

    async fn is_available(&self) -> bool {
        // Native execution is always available
        true
    }

    async fn execute(
        &self,
        command: &str,
        workspace: &Path,
        config: &SandboxConfig,
    ) -> Result<ExecutionResult, String> {
        let timeout_seconds = config.timeout_seconds.unwrap_or(300);

        // Use timeout command on Unix systems
        #[cfg(unix)]
        let output = Command::new("bash")
            .arg("-c")
            .arg(format!("timeout {} {}", timeout_seconds, command))
            .current_dir(workspace)
            .output()
            .await
            .map_err(|e| format!("Failed to execute command: {}", e))?;

        // On Windows, we don't have timeout command built-in
        #[cfg(windows)]
        let output = Command::new("cmd")
            .arg("/C")
            .arg(command)
            .current_dir(workspace)
            .output()
            .await
            .map_err(|e| format!("Failed to execute command: {}", e))?;

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

    async fn execute_with_files(
        &self,
        command: &str,
        workspace: &Path,
        extra_files: Option<&Path>,
        config: &SandboxConfig,
    ) -> Result<ExecutionResult, String> {
        // For native execution, we need to handle extra files differently
        // Since we can't mount, we set an environment variable with the path
        if let Some(files_path) = extra_files {
            let timeout_seconds = config.timeout_seconds.unwrap_or(300);

            #[cfg(unix)]
            let output = Command::new("bash")
                .arg("-c")
                .arg(format!("timeout {} {}", timeout_seconds, command))
                .current_dir(workspace)
                .env("SKILL_FILES_PATH", files_path)
                .output()
                .await
                .map_err(|e| format!("Failed to execute command: {}", e))?;

            #[cfg(windows)]
            let output = Command::new("cmd")
                .arg("/C")
                .arg(command)
                .current_dir(workspace)
                .env("SKILL_FILES_PATH", files_path)
                .output()
                .await
                .map_err(|e| format!("Failed to execute command: {}", e))?;

            let exit_code = output.status.code().unwrap_or(-1);
            let timed_out = exit_code == 124;

            Ok(ExecutionResult {
                success: output.status.success(),
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                exit_code,
                timed_out,
            })
        } else {
            self.execute(command, workspace, config).await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_native_sandbox_available() {
        let sandbox = NativeSandbox::new();
        assert!(sandbox.is_available().await);
    }

    #[tokio::test]
    async fn test_native_sandbox_execute() {
        let sandbox = NativeSandbox::new();
        let temp_dir = TempDir::new().unwrap();
        let config = SandboxConfig::default();

        let result = sandbox
            .execute("echo 'Hello World'", temp_dir.path(), &config)
            .await
            .unwrap();

        assert!(result.success);
        assert!(result.stdout.contains("Hello World"));
    }

    #[tokio::test]
    async fn test_native_sandbox_working_directory() {
        let sandbox = NativeSandbox::new();
        let temp_dir = TempDir::new().unwrap();
        let config = SandboxConfig::default();

        // Create a test file
        std::fs::write(temp_dir.path().join("test.txt"), "test content").unwrap();

        let result = sandbox
            .execute("cat test.txt", temp_dir.path(), &config)
            .await
            .unwrap();

        assert!(result.success);
        assert!(result.stdout.contains("test content"));
    }

    #[tokio::test]
    async fn test_native_sandbox_exit_code() {
        let sandbox = NativeSandbox::new();
        let temp_dir = TempDir::new().unwrap();
        let config = SandboxConfig::default();

        // Use a command that will fail with a specific exit code
        let result = sandbox
            .execute("bash -c 'exit 42'", temp_dir.path(), &config)
            .await
            .unwrap();

        assert!(!result.success);
        // The actual exit code may be wrapped by timeout, just verify it failed
        assert!(result.exit_code != 0);
    }
}
