//! Scope enforcement module for restricting agent operations to specific workspaces

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Scope type for an agent
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ScopeType {
    /// Global scope - agent can access any path
    Global,
    /// Workspace scope - agent can only access paths within workspace
    Workspace,
}

impl Default for ScopeType {
    fn default() -> Self {
        ScopeType::Global
    }
}

impl From<&str> for ScopeType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "workspace" => ScopeType::Workspace,
            _ => ScopeType::Global,
        }
    }
}

impl From<Option<String>> for ScopeType {
    fn from(s: Option<String>) -> Self {
        s.as_deref().map(ScopeType::from).unwrap_or_default()
    }
}

/// Scope enforcer for validating paths and commands against workspace boundaries
#[derive(Debug, Clone)]
pub struct ScopeEnforcer {
    scope_type: ScopeType,
    workspace_path: Option<PathBuf>,
}

impl Default for ScopeEnforcer {
    fn default() -> Self {
        Self::global()
    }
}

impl ScopeEnforcer {
    /// Create a global scope enforcer (allows everything)
    pub fn global() -> Self {
        Self {
            scope_type: ScopeType::Global,
            workspace_path: None,
        }
    }

    /// Create a workspace-scoped enforcer
    pub fn workspace(workspace_path: PathBuf) -> Self {
        Self {
            scope_type: ScopeType::Workspace,
            workspace_path: Some(workspace_path),
        }
    }

    /// Create from scope type and optional workspace path
    pub fn new(scope_type: ScopeType, workspace_path: Option<PathBuf>) -> Self {
        Self {
            scope_type,
            workspace_path,
        }
    }

    /// Check if a path is allowed under current scope
    pub fn is_path_allowed(&self, path: &Path) -> bool {
        match self.scope_type {
            ScopeType::Global => true,
            ScopeType::Workspace => {
                if let Some(ref workspace) = self.workspace_path {
                    // Canonicalize paths for comparison
                    let canonical_workspace = match workspace.canonicalize() {
                        Ok(p) => p,
                        Err(_) => return false, // Workspace doesn't exist
                    };

                    let canonical_path = match path.canonicalize() {
                        Ok(p) => p,
                        Err(_) => {
                            // Path doesn't exist yet, check if parent is within workspace
                            if let Some(parent) = path.parent() {
                                if let Ok(canonical_parent) = parent.canonicalize() {
                                    return canonical_parent.starts_with(&canonical_workspace);
                                }
                            }
                            // For new paths, check if the non-canonicalized path starts with workspace
                            return path.starts_with(workspace);
                        }
                    };

                    canonical_path.starts_with(&canonical_workspace)
                } else {
                    false // No workspace set, deny all
                }
            }
        }
    }

    /// Validate a shell command for potential path escapes
    /// Returns Ok(()) if command is safe, Err with reason if not
    pub fn validate_command(&self, command: &str) -> Result<(), String> {
        match self.scope_type {
            ScopeType::Global => Ok(()),
            ScopeType::Workspace => {
                let workspace = self
                    .workspace_path
                    .as_ref()
                    .ok_or("No workspace path set for workspace scope")?;

                // Check for dangerous patterns
                let dangerous_patterns = [
                    "cd /",     // Absolute path navigation
                    "cd ~",     // Home directory navigation
                    "cd ..",    // Parent directory navigation (could escape)
                    "rm -rf /", // Dangerous root deletion
                    "rm -rf ~", // Dangerous home deletion
                    "> /",      // Writing to absolute paths
                    ">> /",     // Appending to absolute paths
                ];

                for pattern in dangerous_patterns {
                    if command.contains(pattern) {
                        return Err(format!(
                            "Command contains potentially dangerous pattern '{}' that may escape workspace",
                            pattern
                        ));
                    }
                }

                // Check for absolute paths outside workspace
                // This is a basic check - more sophisticated parsing would be needed for production
                let parts: Vec<&str> = command.split_whitespace().collect();
                for part in parts {
                    if part.starts_with('/') {
                        let path = PathBuf::from(part);
                        if !self.is_path_allowed(&path) {
                            return Err(format!(
                                "Command references path '{}' outside workspace '{}'",
                                part,
                                workspace.display()
                            ));
                        }
                    }
                }

                Ok(())
            }
        }
    }

    /// Get the workspace path if set
    pub fn workspace_path(&self) -> Option<&PathBuf> {
        self.workspace_path.as_ref()
    }

    /// Get the scope type
    pub fn scope_type(&self) -> &ScopeType {
        &self.scope_type
    }

    /// Check if this is a workspace scope
    pub fn is_workspace_scope(&self) -> bool {
        self.scope_type == ScopeType::Workspace
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_global_scope_allows_all() {
        let enforcer = ScopeEnforcer::global();
        assert!(enforcer.is_path_allowed(Path::new("/etc/passwd")));
        assert!(enforcer.is_path_allowed(Path::new("/home/user/file.txt")));
        assert!(enforcer.is_path_allowed(Path::new("/tmp/test")));
    }

    #[test]
    fn test_workspace_scope_restricts_paths() {
        let workspace = TempDir::new().unwrap();
        let enforcer = ScopeEnforcer::workspace(workspace.path().to_path_buf());

        // Create a file in workspace for testing
        let allowed_file = workspace.path().join("test.txt");
        std::fs::write(&allowed_file, "test").unwrap();

        assert!(enforcer.is_path_allowed(&allowed_file));
        assert!(!enforcer.is_path_allowed(Path::new("/etc/passwd")));
        assert!(!enforcer.is_path_allowed(Path::new("/tmp/outside")));
    }

    #[test]
    fn test_validate_command_global() {
        let enforcer = ScopeEnforcer::global();
        assert!(enforcer.validate_command("rm -rf /").is_ok()); // Global allows dangerous commands
    }

    #[test]
    fn test_validate_command_workspace() {
        let workspace = TempDir::new().unwrap();
        let enforcer = ScopeEnforcer::workspace(workspace.path().to_path_buf());

        // Should block dangerous patterns
        assert!(enforcer.validate_command("cd /").is_err());
        assert!(enforcer.validate_command("rm -rf /").is_err());

        // Should allow safe commands
        assert!(enforcer.validate_command("ls").is_ok());
        assert!(enforcer.validate_command("cat file.txt").is_ok());
    }

    #[test]
    fn test_scope_type_from_string() {
        assert_eq!(ScopeType::from("global"), ScopeType::Global);
        assert_eq!(ScopeType::from("workspace"), ScopeType::Workspace);
        assert_eq!(ScopeType::from("unknown"), ScopeType::Global);
    }
}
