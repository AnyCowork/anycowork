use crate::permissions::{PermissionRequest, PermissionType};
use crate::tools::{Tool, ToolContext};
use async_trait::async_trait;
use serde_json::{json, Value};
use tokio::process::Command;

use tauri::Runtime;

pub struct BashTool;

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

        let output = Command::new("bash")
            .arg("-c")
            .arg(command)
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_verify_result() {
        let tool: Box<dyn Tool<tauri::test::MockRuntime>> = Box::new(BashTool);

        let res_ok = json!({"exit_code": 0});
        assert!(tool.verify_result(&res_ok));

        let res_fail = json!({"exit_code": 1});
        assert!(!tool.verify_result(&res_fail));
    }

    #[test]
    fn test_needs_summarization() {
        let tool: Box<dyn Tool<tauri::test::MockRuntime>> = Box::new(BashTool);
        assert!(tool.needs_summarization(&json!({}), &json!({})));
    }
}
