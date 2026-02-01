//! Sandbox abstraction for secure command execution

mod docker;
mod native;
mod traits;

pub use docker::DockerSandbox;
pub use native::NativeSandbox;
pub use traits::{ExecutionResult, Sandbox, SandboxConfig};

/// Create a sandbox based on the execution mode and availability
pub async fn create_sandbox(
    mode: &crate::config::ExecutionMode,
) -> Result<Box<dyn Sandbox>, String> {
    match mode {
        crate::config::ExecutionMode::Sandbox => {
            let sandbox = DockerSandbox::new().await?;
            if !sandbox.is_available().await {
                return Err("Docker is required for sandbox mode but is not available".to_string());
            }
            Ok(Box::new(sandbox))
        }
        crate::config::ExecutionMode::Direct => Ok(Box::new(NativeSandbox::new())),
        crate::config::ExecutionMode::Flexible => {
            let docker = DockerSandbox::new().await;
            if let Ok(sandbox) = docker {
                if sandbox.is_available().await {
                    return Ok(Box::new(sandbox));
                }
            }
            Ok(Box::new(NativeSandbox::new()))
        }
    }
}
