//! Search tool for finding text patterns in files

use super::{AnyCoworkTool, ToolError};
use crate::permissions::{PermissionManager, PermissionRequest, PermissionType};
use crate::sandbox::{Sandbox, SandboxConfig};
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Deserialize, JsonSchema)]
pub struct SearchArgs {
    /// The text or regex pattern to search for
    pub query: String,
    /// Relative path to search in (directory or file). Defaults to current directory.
    pub path: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct SearchOutput(serde_json::Value);

/// Tool for searching files
pub struct SearchTool {
    /// Workspace path
    pub workspace_path: PathBuf,
    /// Permission manager
    pub permissions: Arc<PermissionManager>,
    /// Sandbox for execution
    pub sandbox: Arc<dyn Sandbox>,
}

impl SearchTool {
    /// Create a new search tool
    pub fn new(
        workspace_path: PathBuf,
        permissions: Arc<PermissionManager>,
        sandbox: Arc<dyn Sandbox>,
    ) -> Self {
        Self {
            workspace_path,
            permissions,
            sandbox,
        }
    }

    fn validate_path(&self, path_str: &str) -> Result<(), ToolError> {
        if path_str.contains("..") || path_str.starts_with('/') {
            return Err(ToolError::validation_failed(
                "Paths must be relative and cannot contain '..'",
            ));
        }
        Ok(())
    }
}

impl Tool for SearchTool {
    const NAME: &'static str = "search_files";

    type Error = ToolError;
    type Args = SearchArgs;
    type Output = SearchOutput;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Search for text patterns in files within the workspace. Uses grep recursively.".to_string(),
            parameters: serde_json::to_value(schemars::schema_for!(SearchArgs)).unwrap(),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let query = args.query;
        let path_str = args.path.as_deref().unwrap_or(".");
        
        self.validate_path(path_str)?;

        // Request permission
        let perm_req = PermissionRequest::new(
            PermissionType::FilesystemRead,
            format!("Agent wants to search files in {}", path_str),
        )
        // .with_session_id(&ctx.session_id)
        .with_resource(path_str)
        .with_metadata("operation", "search");

        let allowed = self
            .permissions
            .check(perm_req)
            .await
            .map_err(ToolError::other)?;

        if !allowed {
            return Err(ToolError::permission_denied("User denied permission"));
        }

        // Escape the query for shell safety
        let escaped_query = query.replace('\'', "'\\''");
        let command = format!("grep -r -n '{}' {}", escaped_query, path_str);

        let config = SandboxConfig::default().with_timeout(60);

        let result = self
            .sandbox
            .execute(&command, &self.workspace_path, &config)
            .await
            .map_err(ToolError::execution_failed)?;

        // grep returns exit code 1 if no matches found
        if result.exit_code == 1 && result.stderr.is_empty() {
            return Ok(SearchOutput(json!("No matches found.")));
        }

        if !result.success && result.exit_code != 1 {
            return Err(ToolError::execution_failed(format!(
                "grep failed: {}",
                result.stderr
            )));
        }

        if result.stdout.is_empty() {
            Ok(SearchOutput(json!("No matches found.")))
        } else {
            Ok(SearchOutput(json!(result.stdout)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::permissions::AllowAllHandler;
    use crate::sandbox::NativeSandbox;

    #[tokio::test]
    async fn test_validate_args() {
        let permissions = Arc::new(PermissionManager::new(AllowAllHandler));
        let sandbox = Arc::new(NativeSandbox::new());
        let tool = SearchTool::new(PathBuf::from("."), permissions, sandbox);

        // Valid path
        assert!(tool.validate_path("src").is_ok());

        // Path traversal
        assert!(tool.validate_path("../src").is_err());

        // Absolute path
        assert!(tool.validate_path("/etc").is_err());
    }
}

#[async_trait::async_trait]
impl AnyCoworkTool for SearchTool {}
