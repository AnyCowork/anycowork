//! Permission handler trait for platform adapters

use super::{PermissionRequest, PermissionResponse};
use async_trait::async_trait;

/// Trait for platform adapters to implement permission handling
///
/// This allows different platforms (Tauri, CLI, Server) to implement
/// their own permission request UI/flow.
#[async_trait]
pub trait PermissionHandler: Send + Sync {
    /// Request permission from the user
    ///
    /// # Arguments
    /// * `request` - The permission request containing details about what's being requested
    ///
    /// # Returns
    /// * `Ok(PermissionResponse)` - The user's response
    /// * `Err(String)` - An error occurred while requesting permission
    async fn request_permission(
        &self,
        request: PermissionRequest,
    ) -> Result<PermissionResponse, String>;
}

/// A permission handler that always allows
#[derive(Debug, Default)]
pub struct AllowAllHandler;

#[async_trait]
impl PermissionHandler for AllowAllHandler {
    async fn request_permission(
        &self,
        _request: PermissionRequest,
    ) -> Result<PermissionResponse, String> {
        Ok(PermissionResponse::Allow)
    }
}

/// A permission handler that always denies
#[derive(Debug, Default)]
pub struct DenyAllHandler;

#[async_trait]
impl PermissionHandler for DenyAllHandler {
    async fn request_permission(
        &self,
        _request: PermissionRequest,
    ) -> Result<PermissionResponse, String> {
        Ok(PermissionResponse::Deny)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::permissions::PermissionType;

    #[tokio::test]
    async fn test_allow_all_handler() {
        let handler = AllowAllHandler;
        let request = PermissionRequest::new(PermissionType::ShellExecute, "Test command");
        let response = handler.request_permission(request).await.unwrap();
        assert!(response.is_allowed());
    }

    #[tokio::test]
    async fn test_deny_all_handler() {
        let handler = DenyAllHandler;
        let request = PermissionRequest::new(PermissionType::ShellExecute, "Test command");
        let response = handler.request_permission(request).await.unwrap();
        assert!(!response.is_allowed());
    }
}
