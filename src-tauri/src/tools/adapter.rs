//! Adapter to bridge anycowork_core tools with Tauri runtime

use crate::permissions::PermissionManager as TauriPermissionManager;
use crate::tools::{Tool as TauriTool, ToolContext as TauriToolContext};
use anycowork_core::permissions::{
    DenyAllHandler, PermissionHandler, PermissionManager, PermissionRequest, PermissionResponse,
};
use anycowork_core::sandbox::{NativeSandbox, Sandbox};
use anycowork_core::tools::AnyCoworkTool;
// use rig::tool::Tool as RigTool; // Replaced by AnyCoworkTool which extends RigTool
use std::path::PathBuf;
use std::sync::Arc;
use tauri::Runtime;

/// Permission handler that bridges to Tauri's permission system
pub struct TauriBridgePermissionHandler<R: Runtime> {
    tauri_permissions: Arc<TauriPermissionManager>,
    window: Option<tauri::WebviewWindow<R>>,
    session_id: String,
}

impl<R: Runtime> TauriBridgePermissionHandler<R> {
    pub fn new(
        tauri_permissions: Arc<TauriPermissionManager>,
        window: Option<tauri::WebviewWindow<R>>,
        session_id: String,
    ) -> Self {
        Self {
            tauri_permissions,
            window,
            session_id,
        }
    }
}

#[async_trait::async_trait]
impl<R: Runtime> PermissionHandler for TauriBridgePermissionHandler<R> {
    async fn request_permission(
        &self,
        request: PermissionRequest,
    ) -> Result<PermissionResponse, String> {
        // Convert core PermissionRequest to Tauri PermissionRequest
        let tauri_request = crate::permissions::PermissionRequest {
            id: request.id.clone(),
            permission_type: match request.permission_type {
                anycowork_core::permissions::PermissionType::FilesystemRead => {
                    crate::permissions::PermissionType::FilesystemRead
                }
                anycowork_core::permissions::PermissionType::FilesystemWrite => {
                    crate::permissions::PermissionType::FilesystemWrite
                }
                anycowork_core::permissions::PermissionType::ShellExecute => {
                    crate::permissions::PermissionType::ShellExecute
                }
                anycowork_core::permissions::PermissionType::Network => {
                    crate::permissions::PermissionType::Network
                }
                anycowork_core::permissions::PermissionType::Unknown => {
                    crate::permissions::PermissionType::Unknown
                }
            },
            message: request.message.clone(),
            metadata: {
                let mut map = request.metadata.clone();
                map.insert("session_id".to_string(), self.session_id.clone());
                map
            },
        };

        let allowed = self
            .tauri_permissions
            .request_permission(self.window.as_ref(), tauri_request)
            .await?;

        if allowed {
            Ok(PermissionResponse::Allow)
        } else {
            Ok(PermissionResponse::Deny)
        }
    }
}

/// Generic adapter for Rig tools that need to be constructed per-request
pub struct RigToolAdapter<R, T, F>
where
    R: Runtime,
    T: AnyCoworkTool,
    F: Fn(&TauriToolContext<R>) -> T + Send + Sync,
{
    name: String,
    description: String,
    schema: serde_json::Value,
    factory: F,
    dummy_tool: Arc<T>,
    _phantom: std::marker::PhantomData<fn(R)>,
}

impl<R, T, F> RigToolAdapter<R, T, F>
where
    R: Runtime,
    T: AnyCoworkTool,
    F: Fn(&TauriToolContext<R>) -> T + Send + Sync,
{
    /// Create a new adapter. Requires a dummy instance to extract metadata.
    pub async fn new(dummy_tool: T, factory: F) -> Self {
        let definition = dummy_tool.definition("".to_string()).await;
        Self {
            name: definition.name,
            description: definition.description,
            schema: definition.parameters,
            factory,
            dummy_tool: Arc::new(dummy_tool),
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait::async_trait]
impl<R, T, F> TauriTool<R> for RigToolAdapter<R, T, F>
where
    R: Runtime,
    T: AnyCoworkTool + 'static,
    T::Args: Send + Sync + serde::de::DeserializeOwned,
    T::Output: Send + Sync + serde::de::DeserializeOwned, // AnyCoworkTool output must be safe
    F: Fn(&TauriToolContext<R>) -> T + Send + Sync,
{
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn parameters_schema(&self) -> serde_json::Value {
        self.schema.clone()
    }

    async fn execute(
        &self,
        args: serde_json::Value,
        ctx: &TauriToolContext<R>,
    ) -> Result<serde_json::Value, String> {
        let tool = (self.factory)(ctx);
        let typed_args: T::Args = serde_json::from_value(args)
            .map_err(|e| format!("Invalid arguments: {}", e))?;

        let result = tool.call(typed_args).await.map_err(|e| e.to_string())?;

        serde_json::to_value(result).map_err(|e| e.to_string())
    }

    fn verify_result(&self, _result: &serde_json::Value) -> bool {
        // AnyCoworkTool doesn't have verify_result yet, but assuming default is true/safe.
        // If we want to support it, we should add it to AnyCoworkTool.
        true 
    }

    fn needs_summarization(&self, args: &serde_json::Value, result: &serde_json::Value) -> bool {
        let typed_args: Result<T::Args, _> = serde_json::from_value(args.clone());
        if let Ok(args) = typed_args {
             let typed_result: Result<T::Output, _> = serde_json::from_value(result.clone());
             if let Ok(res) = typed_result {
                 self.dummy_tool.needs_summarization(&args, &res)
             } else {
                 false
             }
        } else {
            false
        }
    }

    fn requires_approval(&self, args: &serde_json::Value) -> bool {
         let typed_args: Result<T::Args, _> = serde_json::from_value(args.clone());
         if let Ok(args) = typed_args {
             self.dummy_tool.requires_approval(&args)
         } else {
             false
         }
    }
}

/// Helper to create dependencies (Permissions, Sandbox) from context
fn create_core_deps<R: Runtime + Send + Sync>(
    ctx: &TauriToolContext<R>,
    _workspace_path: PathBuf,
) -> (Arc<PermissionManager>, Arc<dyn Sandbox>) {
    let bridge_handler = TauriBridgePermissionHandler::new(
        ctx.permissions.clone(),
        ctx.window.clone(),
        ctx.session_id.clone(),
    );
    let permissions = Arc::new(PermissionManager::new(bridge_handler));
    
    // Using NativeSandbox for now. In the future, we can support Docker based on config.
    let sandbox = Arc::new(NativeSandbox::new()); // TODO: Reuse sandbox or configure it properly

    (permissions, sandbox)
}

/// Create default tools using Rig implementations wrapped in adapters
pub async fn create_default_tools<R: Runtime + Send + Sync + 'static>(
    workspace_path: PathBuf,
    _execution_mode: &str,
) -> Vec<Box<dyn TauriTool<R>>> {
    let mut tools: Vec<Box<dyn TauriTool<R>>> = Vec::new();

    // 1. Filesystem Tool
    {
        let ws = workspace_path.clone();
        let dummy = anycowork_core::tools::FilesystemTool::new(
            ws.clone(),
            Arc::new(PermissionManager::new(DenyAllHandler)),
        );
        let adapter = RigToolAdapter::new(dummy, move |ctx| {
            let (permissions, _) = create_core_deps(ctx, ws.clone());
            anycowork_core::tools::FilesystemTool::new(ws.clone(), permissions)
        })
        .await;
        tools.push(Box::new(adapter));
    }

    // 2. Search Tool
    {
        let ws = workspace_path.clone();
        let dummy = anycowork_core::tools::SearchTool::new(
            ws.clone(),
            Arc::new(PermissionManager::new(DenyAllHandler)),
            Arc::new(NativeSandbox::new()),
        );
        let adapter = RigToolAdapter::new(dummy, move |ctx| {
            let (permissions, sandbox) = create_core_deps(ctx, ws.clone());
            anycowork_core::tools::SearchTool::new(ws.clone(), permissions, sandbox)
        })
        .await;
        tools.push(Box::new(adapter));
    }

    // 3. Bash Tool
    {
        let ws = workspace_path.clone();
        let dummy = anycowork_core::tools::BashTool::new(
            ws.clone(),
            anycowork_core::config::ExecutionMode::Flexible,
            Arc::new(PermissionManager::new(DenyAllHandler)),
            Arc::new(NativeSandbox::new()),
        );
        let adapter = RigToolAdapter::new(dummy, move |ctx| {
            let (permissions, sandbox) = create_core_deps(ctx, ws.clone());
            anycowork_core::tools::BashTool::new(
                ws.clone(),
                anycowork_core::config::ExecutionMode::Flexible,
                permissions,
                sandbox,
            )
        })
        .await;
        tools.push(Box::new(adapter));
    }

    // 4. Office Tool
    {
        let ws = workspace_path.clone();
        let dummy = anycowork_core::tools::OfficeTool::new(
            ws.clone(),
            Arc::new(PermissionManager::new(DenyAllHandler)),
        );
        let adapter = RigToolAdapter::new(dummy, move |ctx| {
            let (permissions, _) = create_core_deps(ctx, ws.clone());
            anycowork_core::tools::OfficeTool::new(ws.clone(), permissions)
        })
        .await;
        tools.push(Box::new(adapter));
    }

    tools
}
