use anyagents::events::AgentObserver;
use serde_json::Value;
use tauri::{Emitter, Runtime, WebviewWindow};

#[derive(Clone)]
pub struct TauriAgentObserver<R: Runtime> {
    pub window: WebviewWindow<R>,
}

impl<R: Runtime> AgentObserver for TauriAgentObserver<R> {
    fn emit(&self, event_name: &str, payload: Value) -> Result<(), String> {
        log::info!("TauriAgentObserver: Emitting event '{}'", event_name);
        self.window
            .emit(event_name, payload)
            .map_err(|e| e.to_string())
    }
}
