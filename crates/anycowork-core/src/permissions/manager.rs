//! Permission manager with caching

use super::{PermissionHandler, PermissionRequest, PermissionResponse};
use dashmap::DashMap;
use std::sync::Arc;

/// Permission manager that handles caching and delegates to a handler
pub struct PermissionManager {
    /// The platform-specific permission handler
    handler: Arc<dyn PermissionHandler>,
    /// Cache of approved permissions (key -> allowed)
    cache: DashMap<String, bool>,
}

impl PermissionManager {
    /// Create a new permission manager with the given handler
    pub fn new(handler: impl PermissionHandler + 'static) -> Self {
        Self {
            handler: Arc::new(handler),
            cache: DashMap::new(),
        }
    }

    /// Create a permission manager from an Arc handler
    pub fn from_arc(handler: Arc<dyn PermissionHandler>) -> Self {
        Self {
            handler,
            cache: DashMap::new(),
        }
    }

    /// Check if a permission is granted
    ///
    /// First checks the cache, then delegates to the handler if not cached.
    pub async fn check(&self, request: PermissionRequest) -> Result<bool, String> {
        let cache_key = request.cache_key();

        // Check cache first
        if let Some(cached) = self.cache.get(&cache_key) {
            return Ok(*cached);
        }

        // Request from handler
        let response = self.handler.request_permission(request).await?;

        // Cache if appropriate
        if response.should_cache() {
            self.cache.insert(cache_key, true);
        }

        Ok(response.is_allowed())
    }

    /// Directly request permission without caching
    pub async fn request(&self, request: PermissionRequest) -> Result<PermissionResponse, String> {
        self.handler.request_permission(request).await
    }

    /// Clear the permission cache
    pub fn clear_cache(&self) {
        self.cache.clear();
    }

    /// Clear cached permissions for a specific session
    pub fn clear_session_cache(&self, session_id: &str) {
        let prefix = format!("{}:", session_id);
        self.cache.retain(|key, _| !key.starts_with(&prefix));
    }

    /// Pre-approve a permission (add to cache)
    pub fn pre_approve(&self, cache_key: impl Into<String>) {
        self.cache.insert(cache_key.into(), true);
    }

    /// Get the number of cached permissions
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::permissions::{AllowAllHandler, DenyAllHandler, PermissionType};

    #[tokio::test]
    async fn test_permission_manager_allow() {
        let manager = PermissionManager::new(AllowAllHandler);
        let request = PermissionRequest::new(PermissionType::ShellExecute, "test");
        assert!(manager.check(request).await.unwrap());
    }

    #[tokio::test]
    async fn test_permission_manager_deny() {
        let manager = PermissionManager::new(DenyAllHandler);
        let request = PermissionRequest::new(PermissionType::ShellExecute, "test");
        assert!(!manager.check(request).await.unwrap());
    }

    #[tokio::test]
    async fn test_permission_manager_cache() {
        let manager = PermissionManager::new(AllowAllHandler);

        // Pre-approve a permission
        manager.pre_approve("session1:shell_execute:test");

        // This should use the cache
        let request = PermissionRequest::new(PermissionType::ShellExecute, "test")
            .with_session_id("session1")
            .with_resource("test");

        assert!(manager.check(request).await.unwrap());
        assert_eq!(manager.cache_size(), 1);
    }

    #[tokio::test]
    async fn test_clear_session_cache() {
        let manager = PermissionManager::new(AllowAllHandler);

        manager.pre_approve("session1:shell_execute:cmd1");
        manager.pre_approve("session1:shell_execute:cmd2");
        manager.pre_approve("session2:shell_execute:cmd1");

        assert_eq!(manager.cache_size(), 3);

        manager.clear_session_cache("session1");

        assert_eq!(manager.cache_size(), 1);
    }
}
