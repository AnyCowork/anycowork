//! Tool error types

use thiserror::Error;

/// Errors that can occur during tool execution
#[derive(Debug, Error)]
pub enum ToolError {
    /// Missing required argument
    #[error("Missing required argument: {0}")]
    MissingArgument(String),

    /// Invalid argument value
    #[error("Invalid argument '{name}': {reason}")]
    InvalidArgument { name: String, reason: String },

    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Execution failed
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    /// Validation failed
    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Other error
    #[error("{0}")]
    Other(String),
}

impl ToolError {
    /// Create a missing argument error
    pub fn missing_argument(name: impl Into<String>) -> Self {
        Self::MissingArgument(name.into())
    }

    /// Create an invalid argument error
    pub fn invalid_argument(name: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidArgument {
            name: name.into(),
            reason: reason.into(),
        }
    }

    /// Create a permission denied error
    pub fn permission_denied(reason: impl Into<String>) -> Self {
        Self::PermissionDenied(reason.into())
    }

    /// Create an execution failed error
    pub fn execution_failed(reason: impl Into<String>) -> Self {
        Self::ExecutionFailed(reason.into())
    }

    /// Create a validation failed error
    pub fn validation_failed(reason: impl Into<String>) -> Self {
        Self::ValidationFailed(reason.into())
    }

    /// Create an other error
    pub fn other(reason: impl Into<String>) -> Self {
        Self::Other(reason.into())
    }
}

impl From<String> for ToolError {
    fn from(s: String) -> Self {
        Self::Other(s)
    }
}

impl From<&str> for ToolError {
    fn from(s: &str) -> Self {
        Self::Other(s.to_string())
    }
}
