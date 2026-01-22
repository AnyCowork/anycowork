pub mod bash;
pub mod filesystem;
pub mod search;

use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;
use crate::permissions::PermissionManager;

use tauri::Runtime;

pub struct ToolContext<R: Runtime> {
    pub permissions: Arc<PermissionManager>,
    pub window: Option<tauri::WebviewWindow<R>>,
    pub session_id: String,
}

#[async_trait]
pub trait Tool<R: Runtime>: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters_schema(&self) -> Value;
    async fn execute(&self, args: Value, ctx: &ToolContext<R>) -> Result<Value, String>;
    fn requires_approval(&self, _args: &Value) -> bool {
        false
    }
}

