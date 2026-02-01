//! Tauri permission handler implementation

use anycowork_core::permissions::{PermissionHandler, PermissionRequest, PermissionResponse};
use async_trait::async_trait;
use dashmap::DashMap;
use std::sync::Arc;
use tauri::{Emitter, Manager, Runtime, WebviewWindow};
use tokio::sync::oneshot;

/// Permission handler that uses Tauri window for permission UI
pub struct TauriPermissionHandler<R: Runtime> {
    /// The Tauri window to emit events to
    window: WebviewWindow<R>,
    /// Pending permission requests (request_id -> response sender)
    pending: Arc<DashMap<String, oneshot::Sender<bool>>>,
}

impl<R: Runtime> TauriPermissionHandler<R> {
    /// Create a new Tauri permission handler
    pub fn new(window: WebviewWindow<R>) -> Self {
        Self {
            window,
            pending: Arc::new(DashMap::new()),
        }
    }

    /// Create with shared pending map (useful for command handlers)
    pub fn with_pending(
        window: WebviewWindow<R>,
        pending: Arc<DashMap<String, oneshot::Sender<bool>>>,
    ) -> Self {
        Self { window, pending }
    }

    /// Approve a pending permission request
    pub fn approve(&self, request_id: &str) -> bool {
        if let Some((_, tx)) = self.pending.remove(request_id) {
            let _ = tx.send(true);
            true
        } else {
            false
        }
    }

    /// Reject a pending permission request
    pub fn reject(&self, request_id: &str) -> bool {
        if let Some((_, tx)) = self.pending.remove(request_id) {
            let _ = tx.send(false);
            true
        } else {
            false
        }
    }

    /// Get the number of pending requests
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    /// Get the pending requests map (for sharing with command handlers)
    pub fn pending_map(&self) -> Arc<DashMap<String, oneshot::Sender<bool>>> {
        self.pending.clone()
    }
}

impl<R: Runtime> Clone for TauriPermissionHandler<R> {
    fn clone(&self) -> Self {
        Self {
            window: self.window.clone(),
            pending: self.pending.clone(),
        }
    }
}

#[async_trait]
impl<R: Runtime + Send + Sync> PermissionHandler for TauriPermissionHandler<R> {
    async fn request_permission(
        &self,
        request: PermissionRequest,
    ) -> Result<PermissionResponse, String> {
        let (tx, rx) = oneshot::channel();
        self.pending.insert(request.id.clone(), tx);

        // Emit to frontend
        if let Some(session_id) = request.metadata.get("session_id") {
            // Emit to session channel
            self.window
                .app_handle()
                .emit(
                    &format!("session:{}", session_id),
                    serde_json::json!({
                        "type": "permission_request",
                        "request": request
                    }),
                )
                .map_err(|e| e.to_string())?;
        } else {
            // Fallback to global
            self.window
                .app_handle()
                .emit("permission_request", &request)
                .map_err(|e| e.to_string())?;
        }

        // Wait for response
        match rx.await {
            Ok(allowed) => {
                if allowed {
                    Ok(PermissionResponse::Allow)
                } else {
                    Ok(PermissionResponse::Deny)
                }
            }
            Err(_) => {
                // Sender dropped (timeout or app close)
                Ok(PermissionResponse::Deny)
            }
        }
    }
}

/// Standalone permission manager that can be used without a window
#[allow(dead_code)]
pub struct StandalonePermissionManager {
    pending: Arc<DashMap<String, oneshot::Sender<bool>>>,
}

#[allow(dead_code)]
impl StandalonePermissionManager {
    pub fn new() -> Self {
        Self {
            pending: Arc::new(DashMap::new()),
        }
    }

    pub fn pending_map(&self) -> Arc<DashMap<String, oneshot::Sender<bool>>> {
        self.pending.clone()
    }

    pub fn approve(&self, request_id: &str) -> bool {
        if let Some((_, tx)) = self.pending.remove(request_id) {
            let _ = tx.send(true);
            true
        } else {
            false
        }
    }

    pub fn reject(&self, request_id: &str) -> bool {
        if let Some((_, tx)) = self.pending.remove(request_id) {
            let _ = tx.send(false);
            true
        } else {
            false
        }
    }
}

impl Default for StandalonePermissionManager {
    fn default() -> Self {
        Self::new()
    }
}
