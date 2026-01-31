use crate::models::SandboxConfig;
use crate::permissions::{PermissionRequest, PermissionType};
use crate::skills::docker::DockerSandbox;
use crate::tools::{Tool, ToolContext};
use async_trait::async_trait;
use serde_json::{json, Value};
use tokio::process::Command;

use tauri::Runtime;

pub struct BashTool {
    pub workspace_path: std::path::PathBuf,
    pub execution_mode: String,
}

impl BashTool {
    pub fn new(workspace_path: std::path::PathBuf, execution_mode: String) -> Self {
        Self { workspace_path, execution_mode }
    }
}

#[async_trait]
impl<R: Runtime> Tool<R> for BashTool {
    fn name(&self) -> &str {
        "bash"
    }

    fn description(&self) -> &str {
        "Execute a bash command. Use this to run shell commands."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The command to execute"
                }
            },
            "required": ["command"]
        })
    }

    fn verify_result(&self, result: &Value) -> bool {
        // Check exit code
        if let Some(code) = result["exit_code"].as_i64() {
            code == 0
        } else {
            true // If no exit code, assume success? Or failure? Bash tool always returns exit_code.
        }
    }

    fn needs_summarization(&self, _args: &Value, _result: &Value) -> bool {
        true
    }

    async fn execute(&self, args: Value, ctx: &ToolContext<R>) -> Result<Value, String> {
        let command = args["command"].as_str().ok_or("Missing command argument")?;

        let perm_req = PermissionRequest {
            id: uuid::Uuid::new_v4().to_string(),
            permission_type: PermissionType::ShellExecute,
            message: format!("Agent wants to run command: {}", command),
            metadata: {
                let mut map = std::collections::HashMap::new();
                map.insert("command".to_string(), command.to_string());
                map.insert("resource".to_string(), command.to_string());
                map.insert("session_id".to_string(), ctx.session_id.clone());
                map
            },
        };

        let allowed = ctx
            .permissions
            .request_permission(ctx.window.as_ref(), perm_req)
            .await?;

        if !allowed {
            return Err("Permission denied by user".to_string());
        }

        // Initialize sandbox to check availability
        let mut sandbox = DockerSandbox::new();
        sandbox.init().await;
        let docker_available = sandbox.is_available();

        // Determine execution mode
        let use_docker = match self.execution_mode.as_str() {
             "sandbox" => {
                 if !docker_available {
                     return Err("Docker is required for Bash (mode: sandbox) but is not available.".to_string());
                 }
                 true
             },
             "direct" => false,
             "flexible" => docker_available,
             _ => docker_available, // Default to flexible
        };

        if use_docker {
            let config = SandboxConfig {
                image: Some("debian:stable-slim".to_string()),
                memory_limit: Some("256m".to_string()),
                cpu_limit: None,
                timeout_seconds: Some(300),
                network_enabled: Some(true), // Allow network for system bash
            };

            println!("Executing Bash via Docker sandbox...");
            let result = sandbox.execute(command, &self.workspace_path, None, &config).await?;
            
            Ok(json!({
                "stdout": result.stdout,
                "stderr": result.stderr,
                "exit_code": result.exit_code
            }))
        } else {
            let output = Command::new("bash")
                .arg("-c")
                .arg(command)
                .current_dir(&self.workspace_path)
                .output()
                .await
                .map_err(|e| e.to_string())?;

            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

            Ok(json!({
                "stdout": stdout,
                "stderr": stderr,
                "exit_code": output.status.code()
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_verify_result() {
        let tool: Box<dyn Tool<tauri::test::MockRuntime>> = Box::new(BashTool::new(std::path::PathBuf::from("."), "flexible".to_string()));

        let res_ok = json!({"exit_code": 0});
        assert!(tool.verify_result(&res_ok));

        let res_fail = json!({"exit_code": 1});
        assert!(!tool.verify_result(&res_fail));
    }

    #[test]
    fn test_needs_summarization() {
        let tool: Box<dyn Tool<tauri::test::MockRuntime>> = Box::new(BashTool::new(std::path::PathBuf::from("."), "flexible".to_string()));
        assert!(tool.needs_summarization(&json!({}), &json!({})));
    }
}
