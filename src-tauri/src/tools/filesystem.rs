use super::{Tool, ToolContext};
use crate::permissions::{PermissionRequest, PermissionType};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use tauri::Runtime;

pub struct FilesystemTool;

#[async_trait]
impl<R: Runtime> Tool<R> for FilesystemTool {
    fn name(&self) -> &str {
        "filesystem"
    }

    fn description(&self) -> &str {
        "Read, write, list files and directories. path must be relative to workspace root."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": ["read_file", "write_file", "list_dir", "make_dir", "delete_file"],
                    "description": "The operation to perform"
                },
                "path": {
                    "type": "string",
                    "description": "The file or directory path"
                },
                "content": {
                    "type": "string",
                    "description": "Content to write (for write_file)"
                }
            },
            "required": ["operation", "path"]
        })
    }
    async fn validate_args(&self, args: &Value) -> Result<(), String> {
        let path_str = args["path"].as_str().ok_or("Missing path")?;
        if path_str.contains("..") || path_str.starts_with("/") {
            return Err(
                "Access denied: Paths must be relative and cannot contain '..'".to_string(),
            );
        }
        Ok(())
    }

    fn needs_summarization(&self, args: &Value, _result: &Value) -> bool {
        // Summarize if reading a file
        if let Some(op) = args["operation"].as_str() {
            return op == "read_file";
        }
        false
    }

    async fn execute(&self, args: Value, ctx: &ToolContext<R>) -> Result<Value, String> {
        let op = args["operation"].as_str().ok_or("Missing operation")?;
        let path_str = args["path"].as_str().ok_or("Missing path")?;

        // Security check moved to validate_args

        // Permission check for write operations
        let requires_approval = matches!(
            op,
            "write_file" | "delete_file" | "make_dir" | "read_file" | "list_dir"
        );

        if requires_approval {
            let perm_req = PermissionRequest {
                id: uuid::Uuid::new_v4().to_string(),
                permission_type: PermissionType::FilesystemWrite,
                message: format!("Agent wants to {} at {}", op.replace("_", " "), path_str),
                metadata: {
                    let mut map = HashMap::new();
                    map.insert("operation".to_string(), op.to_string());
                    map.insert("path".to_string(), path_str.to_string());
                    map.insert("resource".to_string(), path_str.to_string());
                    map.insert("session_id".to_string(), ctx.session_id.clone());
                    map
                },
            };

            if !ctx
                .permissions
                .request_permission(ctx.window.as_ref(), perm_req)
                .await?
            {
                return Err("Permission denied".to_string());
            }
        }

        let root = std::env::current_dir().unwrap_or(PathBuf::from("."));
        let target_path = root.join(path_str);

        match op {
            "read_file" => {
                let content = fs::read_to_string(target_path).map_err(|e| e.to_string())?;
                Ok(json!(content))
            }
            "write_file" => {
                let content = args["content"].as_str().unwrap_or("");
                if let Some(parent) = target_path.parent() {
                    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                }
                fs::write(target_path, content).map_err(|e| e.to_string())?;
                Ok(json!("File written successfully"))
            }
            "list_dir" => {
                let entries = fs::read_dir(target_path).map_err(|e| e.to_string())?;
                let mut items = Vec::new();
                for e in entries.flatten() {
                    let name = e.file_name().to_string_lossy().to_string();
                    let file_type = if e.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                        "directory"
                    } else {
                        "file"
                    };
                    items.push(json!({
                        "name": name,
                        "type": file_type
                    }));
                }
                Ok(json!(items))
            }
            "make_dir" => {
                fs::create_dir_all(target_path).map_err(|e| e.to_string())?;
                Ok(json!("Directory created"))
            }
            "delete_file" => {
                fs::remove_file(target_path).map_err(|e| e.to_string())?;
                Ok(json!("File deleted"))
            }
            _ => Err(format!("Unknown operation: {}", op)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_validate_args() {
        let tool: Box<dyn Tool<tauri::test::MockRuntime>> = Box::new(FilesystemTool);

        // Valid path
        let args = json!({"operation": "list_dir", "path": "src"});
        assert!(tool.validate_args(&args).await.is_ok());

        // Invalid path (traversal)
        let args_bad = json!({"operation": "list_dir", "path": "../src"});
        assert!(tool.validate_args(&args_bad).await.is_err());

        // Invalid path (absolute)
        let args_abs = json!({"operation": "list_dir", "path": "/etc/passwd"});
        assert!(tool.validate_args(&args_abs).await.is_err());
    }

    #[test]
    fn test_needs_summarization() {
        let tool: Box<dyn Tool<tauri::test::MockRuntime>> = Box::new(FilesystemTool);
        let empty = json!({});

        // read_file -> true
        let args_read = json!({"operation": "read_file", "path": "test.txt"});
        assert!(tool.needs_summarization(&args_read, &empty));

        // list_dir -> false
        let args_list = json!({"operation": "list_dir", "path": "."});
        assert!(!tool.needs_summarization(&args_list, &empty));
    }
}
