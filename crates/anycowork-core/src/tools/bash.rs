//! Bash tool for executing shell commands

use super::{AnyCoworkTool, ToolError};
use crate::config::ExecutionMode;
use crate::permissions::{PermissionManager, PermissionRequest, PermissionType};
use crate::sandbox::{Sandbox, SandboxConfig};
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Deserialize, JsonSchema)]
pub struct BashArgs {
    /// The command to execute
    pub command: String,
}

#[derive(Serialize, Deserialize)]
pub struct BashOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

/// Tool for executing bash commands
pub struct BashTool {
    /// Workspace path for command execution
    pub workspace_path: PathBuf,
    /// Execution mode (sandbox, direct, flexible)
    pub execution_mode: ExecutionMode,
    /// Permission manager
    pub permissions: Arc<PermissionManager>,
    /// Sandbox for execution
    pub sandbox: Arc<dyn Sandbox>,
}

impl BashTool {
    /// Create a new bash tool
    pub fn new(
        workspace_path: PathBuf,
        execution_mode: ExecutionMode,
        permissions: Arc<PermissionManager>,
        sandbox: Arc<dyn Sandbox>,
    ) -> Self {
        Self {
            workspace_path,
            execution_mode,
            permissions,
            sandbox,
        }
    }
}

impl Tool for BashTool {
    const NAME: &'static str = "bash";

    type Error = ToolError;
    type Args = BashArgs;
    type Output = BashOutput;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Execute a bash command. Use this to run shell commands.".to_string(),
            parameters: serde_json::to_value(schemars::schema_for!(BashArgs)).unwrap(),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let command = args.command;

        // Request permission
        let perm_req = PermissionRequest::new(
            PermissionType::ShellExecute,
            format!("Agent wants to run command: {}", command),
        )
        // .with_session_id(&ctx.session_id) // TODO: How to get session_id?
        .with_resource(&command);

        let allowed = self
            .permissions
            .check(perm_req)
            .await
            .map_err(ToolError::other)?;

        if !allowed {
            return Err(ToolError::permission_denied("User denied permission"));
        }

        // Execute the command using the sandbox
        let config = SandboxConfig::default()
            .with_network(true) // Allow network for bash commands
            .with_timeout(300);

        let result = self
            .sandbox
            .execute(&command, &self.workspace_path, &config)
            .await
            .map_err(ToolError::execution_failed)?;

        Ok(BashOutput {
            stdout: result.stdout,
            stderr: result.stderr,
            exit_code: result.exit_code,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::permissions::AllowAllHandler;
    use crate::sandbox::NativeSandbox;

    #[tokio::test]
    async fn test_bash_tool() {
        let permissions = Arc::new(PermissionManager::new(AllowAllHandler));
        let sandbox = Arc::new(NativeSandbox::new());
        let tool = BashTool::new(
            PathBuf::from("."),
            ExecutionMode::Flexible,
            permissions,
            sandbox,
        );

        let args = BashArgs {
            command: "echo 'hello'".to_string(),
        };

        let result = tool.call(args).await.unwrap();
        assert_eq!(result.stdout.trim(), "hello");
        assert_eq!(result.exit_code, 0);
    }
}

#[async_trait::async_trait]
impl AnyCoworkTool for BashTool {}
