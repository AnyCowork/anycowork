pub mod bash;
pub mod filesystem;
pub mod search;
pub mod office;

#[cfg(test)]
pub mod workflow_tests;

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
    
    // 1. Validation (Custom logic beyond schema)
    async fn validate_args(&self, _args: &Value) -> Result<(), String> {
        Ok(())
    }

    // 2. Execution
    async fn execute(&self, args: Value, ctx: &ToolContext<R>) -> Result<Value, String>;
    
    // 3. Verification
    fn verify_result(&self, _result: &Value) -> bool {
        true
    }
    
    // 4. Summarization
    fn needs_summarization(&self, _args: &Value, _result: &Value) -> bool {
        false
    }

    fn requires_approval(&self, _args: &Value) -> bool {
        false
    }
}

