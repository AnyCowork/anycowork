//! Permission types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Types of permissions that can be requested
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionType {
    /// Permission to read files
    FilesystemRead,
    /// Permission to write files
    FilesystemWrite,
    /// Permission to execute shell commands
    ShellExecute,
    /// Permission to make network requests
    Network,
    /// Unknown permission type
    Unknown,
}

impl std::fmt::Display for PermissionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PermissionType::FilesystemRead => write!(f, "filesystem_read"),
            PermissionType::FilesystemWrite => write!(f, "filesystem_write"),
            PermissionType::ShellExecute => write!(f, "shell_execute"),
            PermissionType::Network => write!(f, "network"),
            PermissionType::Unknown => write!(f, "unknown"),
        }
    }
}

/// A request for permission to perform an action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRequest {
    /// Unique identifier for this request
    pub id: String,
    /// Type of permission being requested
    pub permission_type: PermissionType,
    /// Human-readable description of what's being requested
    pub message: String,
    /// Additional metadata about the request
    pub metadata: HashMap<String, String>,
}

impl PermissionRequest {
    /// Create a new permission request
    pub fn new(permission_type: PermissionType, message: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            permission_type,
            message: message.into(),
            metadata: HashMap::new(),
        }
    }

    /// Add metadata to the request
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Add session ID to metadata
    pub fn with_session_id(self, session_id: impl Into<String>) -> Self {
        self.with_metadata("session_id", session_id)
    }

    /// Add resource to metadata
    pub fn with_resource(self, resource: impl Into<String>) -> Self {
        self.with_metadata("resource", resource)
    }

    /// Get a cache key for this permission request
    pub fn cache_key(&self) -> String {
        let session_part = self
            .metadata
            .get("session_id")
            .map(|s| format!("{}:", s))
            .unwrap_or_default();
        let resource = self
            .metadata
            .get("resource")
            .cloned()
            .unwrap_or_else(|| "global".to_string());
        format!("{}{}:{}", session_part, self.permission_type, resource)
    }
}

/// Response to a permission request
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionResponse {
    /// Permission granted for this request only
    Allow,
    /// Permission denied
    Deny,
    /// Permission granted and should be cached
    AllowAlways,
}

impl PermissionResponse {
    /// Check if permission was granted
    pub fn is_allowed(&self) -> bool {
        matches!(self, PermissionResponse::Allow | PermissionResponse::AllowAlways)
    }

    /// Check if permission should be cached
    pub fn should_cache(&self) -> bool {
        matches!(self, PermissionResponse::AllowAlways)
    }
}
