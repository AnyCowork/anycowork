pub mod types;

use std::process::Stdio;
use tokio::process::Command;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use std::sync::Arc;
use tokio::sync::Mutex;
use serde_json::json;
use crate::mcp::types::{JsonRpcRequest, JsonRpcResponse, McpInitializeParams, ClientInfo};

pub struct McpClient {
    _process: Arc<Mutex<tokio::process::Child>>,
    stdin: Arc<Mutex<tokio::process::ChildStdin>>,
    stdout_reader: Arc<Mutex<tokio::io::Lines<BufReader<tokio::process::ChildStdout>>>>,
    next_id: Arc<Mutex<i64>>,
}

impl McpClient {
    pub async fn new(command: &str, args: &[String]) -> Result<Self, String> {
        let mut child = Command::new(command)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped()) // Capture stderr to avoid polluting app logs
            .spawn()
            .map_err(|e| format!("Failed to spawn MCP server: {}", e))?;

        let stdin = child.stdin.take().ok_or("Failed to open stdin")?;
        let stdout = child.stdout.take().ok_or("Failed to open stdout")?;
        let reader = BufReader::new(stdout).lines();

        Ok(Self {
            _process: Arc::new(Mutex::new(child)),
            stdin: Arc::new(Mutex::new(stdin)),
            stdout_reader: Arc::new(Mutex::new(reader)),
            next_id: Arc::new(Mutex::new(1)),
        })
    }

    pub async fn send_request(&self, method: &str, params: Option<serde_json::Value>) -> Result<serde_json::Value, String> {
        let id_val;
        {
            let mut id = self.next_id.lock().await;
            id_val = *id;
            *id += 1;
        }

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(id_val)),
            method: method.to_string(),
            params,
        };

        let json_str = serde_json::to_string(&request).map_err(|e| e.to_string())?;
        
        {
            let mut stdin = self.stdin.lock().await;
            stdin.write_all(json_str.as_bytes()).await.map_err(|e| e.to_string())?;
            stdin.write_all(b"\n").await.map_err(|e| e.to_string())?;
            stdin.flush().await.map_err(|e| e.to_string())?;
        }

        // Extremely naive implementation: Response is assumed to be the next line.
        // In reality, we need a message loop to handle notifications, logs, etc.
        // But for "Initial MVP" connection test, this might suffice if protocol is strict.
        // MCP protocol usually sends logs/notifications.
        // We really need a background loop keying on ID.
        // For now, I'll loop reading lines until I find matching ID.
        
        // This lock prevents concurrent requests from working properly unless we're lucky.
        // But for single-threaded usage (one agent step at a time), it might pass.
        let mut reader = self.stdout_reader.lock().await;
        loop {
            let line = reader.next_line().await.map_err(|e| e.to_string())?
                .ok_or("MCP Server closed connection")?;
            
            if let Ok(response) = serde_json::from_str::<JsonRpcResponse>(&line) {
                 if let Some(resp_id) = &response.id {
                     if resp_id.as_i64() == Some(id_val) {
                         if let Some(error) = response.error {
                             return Err(format!("MCP Error {}: {}", error.code, error.message));
                         }
                         return Ok(response.result.unwrap_or(json!(null)));
                     }
                 }
            } else {
                // Log initialization logs or notifications?
                // println!("MCP Ignored: {}", line);
            }
        }
    }

    pub async fn initialize(&self) -> Result<(), String> {
        let params = McpInitializeParams {
            protocol_version: "2024-11-05".to_string(), 
            capabilities: json!({}),
            client_info: ClientInfo {
                name: "AnyCowork".to_string(),
                version: "0.1.0".to_string(),
            },
        };

        let _result = self.send_request("initialize", Some(json!(params))).await?;
        // We should handle the result (server capabilities)
        
        // Send initialized notification
        // Notifications don't have ID
        let notification = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: None,
            method: "notifications/initialized".to_string(),
            params: None,
        };
        let json_str = serde_json::to_string(&notification).map_err(|e| e.to_string())?;
          {
            let mut stdin = self.stdin.lock().await;
            stdin.write_all(json_str.as_bytes()).await.map_err(|e| e.to_string())?;
            stdin.write_all(b"\n").await.map_err(|e| e.to_string())?;
            stdin.flush().await.map_err(|e| e.to_string())?;
        }

        Ok(())
    }

    pub async fn list_tools(&self) -> Result<Vec<crate::mcp::types::McpTool>, String> {
        let result = self.send_request("tools/list", None).await?;
        let tools_result: crate::mcp::types::McpListToolsResult = serde_json::from_value(result)
            .map_err(|e| format!("Failed to parse tools/list result: {}", e))?;
        Ok(tools_result.tools)
    }
}


use std::collections::HashMap;
use crate::tools::{Tool, ToolContext};
use async_trait::async_trait;

use tauri::Runtime;

pub struct ToolRegistry<R: Runtime> {
    tools: HashMap<String, Box<dyn Tool<R>>>,
}

impl<R: Runtime> Default for ToolRegistry<R> {
    fn default() -> Self {
        Self::new()
    }
}

impl<R: Runtime> ToolRegistry<R> {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register(&mut self, tool: Box<dyn Tool<R>>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    pub fn get(&self, name: &str) -> Option<&dyn Tool<R>> {
        self.tools.get(name).map(|b| &**b)
    }

    pub fn list(&self) -> Vec<&dyn Tool<R>> {
        self.tools.values().map(|b| &**b).collect()
    }
}

pub struct McpToolAdapter {
    pub client: Arc<McpClient>,
    pub name: String,
    pub description: String,
    pub schema: serde_json::Value,
}

#[async_trait]
impl<R: Runtime> Tool<R> for McpToolAdapter {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn parameters_schema(&self) -> serde_json::Value {
        self.schema.clone()
    }

    async fn execute(&self, args: serde_json::Value, _ctx: &ToolContext<R>) -> Result<serde_json::Value, String> {
        let params = json!({
            "name": self.name,
            "arguments": args
        });
        
        let result = self.client.send_request("tools/call", Some(params)).await?;
        
        // MCP tools/call result structure: { content: [{type: "text", text: "..."}] }
        // We probably want to return just the text or the raw structure?
        // AgentLoop expects a Value.
        Ok(result)
    }
}

