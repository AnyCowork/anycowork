pub mod scope;

pub use scope::{ScopeEnforcer, ScopeType};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PermissionType {
    FilesystemRead,
    FilesystemWrite,
    ShellExecute,
    Network,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRequest {
    pub id: String,
    pub permission_type: PermissionType,
    pub message: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionResponse {
    Allow,
    Deny,
    AllowAlways,
}

use dashmap::DashMap;
use tokio::sync::oneshot;

pub struct PermissionManager {
    // Map of permission_key -> allowed (true/false)
    cache: Arc<Mutex<HashMap<String, bool>>>,
    // Map of request_id -> sender
    pending_requests: Arc<DashMap<String, oneshot::Sender<bool>>>,
}

impl Default for PermissionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PermissionManager {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            pending_requests: Arc::new(DashMap::new()),
        }
    }

    pub async fn request_permission(
        &self,
        observer: Option<&Arc<dyn crate::events::AgentObserver>>,
        req: PermissionRequest,
    ) -> Result<bool, String> {
        let session_part = req
            .metadata
            .get("session_id")
            .map(|s| format!("{}:", s))
            .unwrap_or_default();
        let key = format!(
            "{}{:?}:{}",
            session_part,
            req.permission_type,
            req.metadata
                .get("resource")
                .unwrap_or(&"global".to_string())
        );

        // Check cache first
        {
            let cache = self.cache.lock().unwrap();
            if let Some(allowed) = cache.get(&key) {
                return Ok(*allowed);
            }
        }

        // If no observer is available, default to deny (safety)
        let observer = match observer {
            Some(o) => o,
            None => return Ok(false),
        };

        // Create response channel
        let (tx, rx) = oneshot::channel();
        self.pending_requests.insert(req.id.clone(), tx);

        // Emit event to frontend
        if let Some(session_id) = req.metadata.get("session_id") {
            log::info!("PermissionManager: Emitting permission_request to session:{}", session_id);
            // Emit to session channel
            observer.emit(
                &format!("session:{}", session_id),
                serde_json::json!({
                    "type": "permission_request",
                    "request": req
                }),
            ).map_err(|e| e.to_string())?;
        } else {
            log::info!("PermissionManager: Emitting global permission_request");
            // Fallback to global
            observer.emit("permission_request", serde_json::to_value(&req).unwrap())
                .map_err(|e| e.to_string())?;
        }

        // Wait for response
        match rx.await {
            Ok(allowed) => {
                if allowed {
                    // Update cache
                    let mut cache = self.cache.lock().unwrap();
                    cache.insert(key, true);
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Err(_) => {
                // Sender dropped (timeout or app close)
                Ok(false)
            }
        }
    }

    pub fn approve_request(&self, request_id: &str) {
        if let Some((_, tx)) = self.pending_requests.remove(request_id) {
            let _ = tx.send(true);
        }
    }

    pub fn reject_request(&self, request_id: &str) {
        if let Some((_, tx)) = self.pending_requests.remove(request_id) {
            let _ = tx.send(false);
        }
    }

    pub fn get_pending_requests(&self) -> Vec<String> {
        self.pending_requests
            .iter()
            .map(|entry| entry.key().clone())
            .collect()
    }
}
