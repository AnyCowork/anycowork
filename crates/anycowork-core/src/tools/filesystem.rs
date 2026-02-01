//! Filesystem tool for file operations

use super::{AnyCoworkTool, ToolError};
use crate::permissions::{PermissionManager, PermissionRequest, PermissionType};
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Deserialize, JsonSchema)]
#[serde(tag = "operation", rename_all = "snake_case")]
pub enum FilesystemArgs {
    ReadFile {
        /// The file path relative to workspace
        path: String,
    },
    WriteFile {
        /// The file path relative to workspace
        path: String,
        /// Content to write
        content: String,
    },
    ListDir {
        /// The directory path relative to workspace
        path: String,
    },
    MakeDir {
        /// The directory path relative to workspace
        path: String,
    },
    DeleteFile {
        /// The file path relative to workspace
        path: String,
    },
}

#[derive(Serialize, Deserialize)]
pub struct FilesystemOutput(serde_json::Value);

/// Tool for filesystem operations
pub struct FilesystemTool {
    /// Workspace path (root for all operations)
    pub workspace_path: PathBuf,
    /// Permission manager
    pub permissions: Arc<PermissionManager>,
}

impl FilesystemTool {
    /// Create a new filesystem tool
    pub fn new(workspace_path: PathBuf, permissions: Arc<PermissionManager>) -> Self {
        Self {
            workspace_path,
            permissions,
        }
    }

    fn validate_path(&self, path_str: &str) -> Result<PathBuf, ToolError> {
        // Security: prevent path traversal
        if path_str.contains("..") || path_str.starts_with('/') {
            return Err(ToolError::validation_failed(
                "Paths must be relative and cannot contain '..'",
            ));
        }
        Ok(self.workspace_path.join(path_str))
    }
}

impl Tool for FilesystemTool {
    const NAME: &'static str = "filesystem";

    type Error = ToolError;
    type Args = FilesystemArgs;
    type Output = FilesystemOutput;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Read, write, list files and directories. Path must be relative to workspace root.".to_string(),
            parameters: serde_json::to_value(schemars::schema_for!(FilesystemArgs)).unwrap(),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let (op, path_str) = match &args {
            FilesystemArgs::ReadFile { path } => ("read_file", path),
            FilesystemArgs::WriteFile { path, .. } => ("write_file", path),
            FilesystemArgs::ListDir { path } => ("list_dir", path),
            FilesystemArgs::MakeDir { path } => ("make_dir", path),
            FilesystemArgs::DeleteFile { path } => ("delete_file", path),
        };

        let target_path = self.validate_path(path_str)?;

        // Determine permission type
        let (permission_type, msg_verb) = match op {
            "read_file" | "list_dir" => (PermissionType::FilesystemRead, "read"),
            "write_file" | "delete_file" | "make_dir" => (PermissionType::FilesystemWrite, "modify"),
            _ => (PermissionType::FilesystemWrite, "access"),
        };

        let perm_req = PermissionRequest::new(
            permission_type,
            format!(
                "Agent wants to {} {} at {}",
                msg_verb,
                if op == "list_dir" { "directory" } else { "file" },
                path_str
            ),
        )
        .with_resource(path_str)
        .with_metadata("operation", op);

        let allowed = self
            .permissions
            .check(perm_req)
            .await
            .map_err(ToolError::other)?;

        if !allowed {
            return Err(ToolError::permission_denied("User denied permission"));
        }

        let result = match args {
            FilesystemArgs::ReadFile { .. } => {
                let content = fs::read_to_string(&target_path)
                    .map_err(|e| ToolError::execution_failed(e.to_string()))?;
                json!(content)
            }
            FilesystemArgs::WriteFile { content, .. } => {
                if let Some(parent) = target_path.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|e| ToolError::execution_failed(e.to_string()))?;
                }
                fs::write(&target_path, content)
                    .map_err(|e| ToolError::execution_failed(e.to_string()))?;
                json!("File written successfully")
            }
            FilesystemArgs::ListDir { .. } => {
                let entries = fs::read_dir(&target_path)
                    .map_err(|e| ToolError::execution_failed(e.to_string()))?;
                let mut items = Vec::new();
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    let file_type = if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                        "directory"
                    } else {
                        "file"
                    };
                    items.push(json!({
                        "name": name,
                        "type": file_type
                    }));
                }
                json!(items)
            }
            FilesystemArgs::MakeDir { .. } => {
                fs::create_dir_all(&target_path)
                    .map_err(|e| ToolError::execution_failed(e.to_string()))?;
                json!("Directory created")
            }
            FilesystemArgs::DeleteFile { .. } => {
                fs::remove_file(&target_path)
                    .map_err(|e| ToolError::execution_failed(e.to_string()))?;
                json!("File deleted")
            }
        };

        Ok(FilesystemOutput(result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::permissions::AllowAllHandler;

    #[tokio::test]
    async fn test_filesystem_tool() {
        let temp_dir = tempfile::tempdir().unwrap();
        let permissions = Arc::new(PermissionManager::new(AllowAllHandler));
        let tool = FilesystemTool::new(temp_dir.path().to_path_buf(), permissions);

        // Write file
        let write_args = FilesystemArgs::WriteFile {
            path: "test.txt".to_string(),
            content: "hello world".to_string(),
        };
        tool.call(write_args).await.unwrap();

        // Read file
        let read_args = FilesystemArgs::ReadFile {
            path: "test.txt".to_string(),
        };
        let result = tool.call(read_args).await.unwrap();
        assert_eq!(result.0.as_str().unwrap(), "hello world");
    }

    #[tokio::test]
    async fn test_security() {
        let temp_dir = tempfile::tempdir().unwrap();
        let permissions = Arc::new(PermissionManager::new(AllowAllHandler));
        let tool = FilesystemTool::new(temp_dir.path().to_path_buf(), permissions);

        let args = FilesystemArgs::ReadFile {
            path: "../outside.txt".to_string(),
        };
        assert!(tool.call(args).await.is_err());
    }
}

#[async_trait::async_trait]
impl AnyCoworkTool for FilesystemTool {
    fn needs_summarization(&self, args: &Self::Args, _result: &Self::Output) -> bool {
        matches!(args, FilesystemArgs::ReadFile { .. })
    }
}
