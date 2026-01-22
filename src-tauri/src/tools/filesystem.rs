use super::{Tool, ToolContext};
use crate::permissions::{PermissionRequest, PermissionType};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::fs;
use std::collections::HashMap;
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

    async fn execute(&self, args: Value, ctx: &ToolContext<R>) -> Result<Value, String> {
        let op = args["operation"].as_str().ok_or("Missing operation")?;
        let path_str = args["path"].as_str().ok_or("Missing path")?;

        // Security: Prevent breaking out of workspace (simplified)
        if path_str.contains("..") || path_str.starts_with("/") {
            return Err("Access denied: Paths must be relative and cannot contain '..'".to_string());
        }
        
        // Permission check for write operations
        let requires_approval = match op {
            "write_file" | "delete_file" | "make_dir" | "read_file" | "list_dir" => true,
            _ => false,
        };

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
            
            if !ctx.permissions.request_permission(ctx.window.as_ref(), perm_req).await? {
                return Err("Permission denied".to_string());
            }
        }

        let root = std::env::current_dir().unwrap_or(PathBuf::from("."));
        let target_path = root.join(path_str);

        match op {
            "read_file" => {
                let content = fs::read_to_string(target_path).map_err(|e| e.to_string())?;
                Ok(json!(content))
            },
            "write_file" => {
                let content = args["content"].as_str().unwrap_or("");
                if let Some(parent) = target_path.parent() {
                    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                }
                fs::write(target_path, content).map_err(|e| e.to_string())?;
                Ok(json!("File written successfully"))
            },
            "list_dir" => {
                let entries = fs::read_dir(target_path).map_err(|e| e.to_string())?;
                let mut names = Vec::new();
                for entry in entries {
                    if let Ok(e) = entry {
                        names.push(e.file_name().to_string_lossy().to_string());
                    }
                }
                Ok(json!(names))
            },
            "make_dir" => {
                 fs::create_dir_all(target_path).map_err(|e| e.to_string())?;
                 Ok(json!("Directory created"))
            },
            "delete_file" => {
                fs::remove_file(target_path).map_err(|e| e.to_string())?;
                Ok(json!("File deleted"))
            }
            _ => Err(format!("Unknown operation: {}", op))
        }
    }
}
