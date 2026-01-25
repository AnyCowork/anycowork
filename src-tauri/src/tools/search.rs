use super::{Tool, ToolContext};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::process::Command;

use tauri::Runtime;

pub struct SearchTool;

#[async_trait]
impl<R: Runtime> Tool<R> for SearchTool {
    fn name(&self) -> &str {
        "search_files"
    }

    fn description(&self) -> &str {
        "Search for text patterns in files within the workspace. Uses grep recursively."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "The text or regex pattern to search for"
                },
                "path": {
                    "type": "string",
                    "description": "Relative path to search in (directory or file). Defaults to current directory."
                }
            },
            "required": ["query"]
        })
    }

    async fn validate_args(&self, args: &Value) -> Result<(), String> {
        let path_str = args["path"].as_str().unwrap_or(".");
        if path_str.contains("..") || path_str.starts_with("/") {
            return Err(
                "Access denied: Paths must be relative and cannot contain '..'".to_string(),
            );
        }
        Ok(())
    }

    fn verify_result(&self, result: &Value) -> bool {
        if let Some(s) = result.as_str() {
            !s.starts_with("Error:")
        } else {
            true
        }
    }

    fn needs_summarization(&self, _args: &Value, result: &Value) -> bool {
        // Summarize if we found valid results (not empty or "No matches")
        if let Some(s) = result.as_str() {
            return s != "No matches found." && !s.starts_with("Error:");
        }
        false
    }

    async fn execute(&self, args: Value, _ctx: &ToolContext<R>) -> Result<Value, String> {
        let query = args["query"].as_str().ok_or("Missing query")?;
        let path_str = args["path"].as_str().unwrap_or(".");

        let root = std::env::current_dir().unwrap_or(PathBuf::from("."));
        let target_path = root.join(path_str);

        // Permission check
        let perm_req = crate::permissions::PermissionRequest {
            id: uuid::Uuid::new_v4().to_string(),
            permission_type: crate::permissions::PermissionType::FilesystemRead,
            message: format!("Agent wants to search files in {}", path_str),
            metadata: {
                let mut map = std::collections::HashMap::new();
                map.insert("operation".to_string(), "search".to_string());
                map.insert("path".to_string(), path_str.to_string());
                map.insert("resource".to_string(), path_str.to_string());
                map
            },
        };

        // Note: We need to handle potential absence of window in ToolContext slightly better or rely on default deny?
        // Method signature of request_permission handles Option<Window>.
        if !_ctx
            .permissions
            .request_permission(_ctx.window.as_ref(), perm_req)
            .await?
        {
            return Err("Permission denied".to_string());
        }

        // Run grep -r "query" target_path
        let output = Command::new("grep")
            .arg("-r")
            .arg("-n") // Line numbers
            .arg(query)
            .arg(target_path)
            .output()
            .map_err(|e| format!("Failed to execute grep: {}", e))?;

        if !output.status.success() {
            // grep returns exit code 1 if no matches found
            if output.status.code() == Some(1) {
                return Ok(json!("No matches found."));
            }
            return Err(format!(
                "grep failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let result = String::from_utf8_lossy(&output.stdout).to_string();
        if result.is_empty() {
            Ok(json!("No matches found."))
        } else {
            Ok(json!(result))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_validate_args() {
        let tool: Box<dyn Tool<tauri::test::MockRuntime>> = Box::new(SearchTool);
        let args_bad = json!({"query": "test", "path": "../src"});
        assert!(tool.validate_args(&args_bad).await.is_err());
    }

    #[test]
    fn test_verify_result() {
        let tool: Box<dyn Tool<tauri::test::MockRuntime>> = Box::new(SearchTool);
        let res_ok = json!("Found matches");
        assert!(tool.verify_result(&res_ok));

        let res_err = json!("Error: grep failed");
        assert!(!tool.verify_result(&res_err));
    }

    #[test]
    fn test_needs_summarization() {
        let tool: Box<dyn Tool<tauri::test::MockRuntime>> = Box::new(SearchTool);
        let empty = json!({});

        // Valid results -> true
        let res_found = json!("file.rs:1: match");
        assert!(tool.needs_summarization(&empty, &res_found));

        // No matches -> false
        let res_none = json!("No matches found.");
        assert!(!tool.needs_summarization(&empty, &res_none));
    }
}
