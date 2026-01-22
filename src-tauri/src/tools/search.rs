use super::{Tool, ToolContext};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::process::Command;
use std::path::PathBuf;

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

    async fn execute(&self, args: Value, _ctx: &ToolContext<R>) -> Result<Value, String> {
        let query = args["query"].as_str().ok_or("Missing query")?;
        let path_str = args["path"].as_str().unwrap_or(".");

        // Security check
        if path_str.contains("..") || path_str.starts_with("/") {
             return Err("Access denied: Paths must be relative and cannot contain '..'".to_string());
        }

        let root = std::env::current_dir().unwrap_or(PathBuf::from("."));
        let target_path = root.join(path_str);

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
             return Err(format!("grep failed: {}", String::from_utf8_lossy(&output.stderr)));
        }

        let result = String::from_utf8_lossy(&output.stdout).to_string();
        if result.is_empty() {
            Ok(json!("No matches found."))
        } else {
            Ok(json!(result))
        }
    }
}
