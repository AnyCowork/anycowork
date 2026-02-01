//! Tauri event bridge for core events

use anycowork_core::events::{AgentEvent, EventChannel};
use std::sync::Arc;
use tauri::{Emitter, Manager, Runtime, WebviewWindow};

/// Bridge that forwards core events to Tauri event system
pub struct TauriEventBridge<R: Runtime> {
    events: Arc<EventChannel>,
    window: WebviewWindow<R>,
    session_id: String,
}

impl<R: Runtime> TauriEventBridge<R> {
    /// Create a new event bridge
    pub fn new(events: Arc<EventChannel>, window: WebviewWindow<R>, session_id: String) -> Self {
        Self {
            events,
            window,
            session_id,
        }
    }

    /// Start the event bridge (spawns a background task)
    pub fn start(self) {
        let mut receiver = self.events.subscribe();
        let window = self.window.clone();
        let session_id = self.session_id.clone();

        tauri::async_runtime::spawn(async move {
            while let Ok(event) = receiver.recv().await {
                let event_name = format!("session:{}", session_id);
                if let Err(e) = window.app_handle().emit(&event_name, &event) {
                    log::error!("Failed to emit event: {}", e);
                }
            }
        });
    }
}

/// Spawn an event bridge that forwards core events to Tauri
pub fn spawn_event_bridge<R: Runtime>(
    events: Arc<EventChannel>,
    window: WebviewWindow<R>,
    session_id: String,
) {
    let bridge = TauriEventBridge::new(events, window, session_id);
    bridge.start();
}

/// Helper to emit a single event to a Tauri window
#[allow(dead_code)]
pub fn emit_event<R: Runtime>(
    window: &WebviewWindow<R>,
    session_id: &str,
    event: &AgentEvent,
) -> Result<(), String> {
    let event_name = format!("session:{}", session_id);
    window
        .app_handle()
        .emit(&event_name, event)
        .map_err(|e| e.to_string())
}

/// Helper to emit events to all windows
#[allow(dead_code)]
pub fn emit_event_to_all<R: Runtime, M: Manager<R> + Emitter<R>>(
    app: &M,
    session_id: &str,
    event: &AgentEvent,
) -> Result<(), String> {
    let event_name = format!("session:{}", session_id);
    app.emit(&event_name, event).map_err(|e| e.to_string())
}
