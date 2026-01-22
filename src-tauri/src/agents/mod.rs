pub mod processor;
pub mod optimizations;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod workflow_tests;

use rig::completion::Prompt;
use rig::providers::openai;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::oneshot;
use crate::models::Agent as DbAgent;
use crate::events::{AgentEvent, ExecutionJob, ExecutionStep};
use uuid::Uuid;
use crate::database::DbPool;
use diesel::prelude::*;
use tauri::Emitter;
use crate::tools::{Tool, ToolContext, filesystem::FilesystemTool, search::SearchTool, bash::BashTool};
use crate::permissions::PermissionManager;
use processor::{StreamProcessor, StreamChunk};


use tauri::Runtime;

pub struct AgentLoop<R: Runtime> {
    pub agent_id: String,
    pub session_id: String,
    pub model: String, 
    pub history: Vec<rig::completion::Message>, 
    pub tools: Vec<Box<dyn Tool<R>>>,
    pub snapshot_manager: crate::snapshots::SnapshotManager,
}

impl<R: Runtime> AgentLoop<R> {
    pub async fn new(agent_db: &DbAgent) -> Self {
        // Register default tools
        let tools: Vec<Box<dyn Tool<R>>> = vec![
            Box::new(FilesystemTool),
            Box::new(SearchTool),
            Box::new(BashTool),
        ];

        Self {
            agent_id: agent_db.id.clone(),
            session_id: "temp".to_string(), // Set later
            model: agent_db.ai_model.clone(),
            history: vec![],
            tools,
            snapshot_manager: crate::snapshots::SnapshotManager::new(std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))),
        }
    }

    pub async fn run(
        &mut self,
        user_message: String,
        window: tauri::WebviewWindow<R>,
        job_id: String,
        _pending_approvals: Arc<dashmap::DashMap<String, oneshot::Sender<bool>>>, // Keeping for backward compat logic if needed, but permissions handle approval now
        permission_manager: Arc<PermissionManager>,
        db_pool: DbPool,
    ) {
        // Initialize job with full fields to match frontend expectation
        let job = ExecutionJob {
            id: job_id.clone(),
            session_id: self.session_id.clone(),
            status: "running".to_string(),
            query: user_message.clone(),
            steps: vec![],
            current_step_index: 0,
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        let _ = Emitter::emit(&window, &format!("session:{}", self.session_id), AgentEvent::JobStarted {
            job: job.clone(),
        });

        // 1. Add User Message to History (Using rig::completion::Message)
        self.history.push(rig::completion::Message { role: "user".to_string(), content: user_message.clone() });

        // Build Rig Agent
        let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| "".to_string());
        if api_key.is_empty() {
             let _ = Emitter::emit(&window, &format!("session:{}", self.session_id), AgentEvent::Token {
                content: "Error: OPENAI_API_KEY environment variable is not set.".to_string(),
            });
             let _ = Emitter::emit(&window, &format!("session:{}", self.session_id), AgentEvent::JobCompleted {
                job: ExecutionJob { status: "failed".to_string(), ..job },
                message: "Missing API Key".to_string(),
            });
            return;
        }

        let client = openai::Client::new(&api_key);
        
        // Prepare tool definitions
        let tools_desc = self.tools.iter().map(|t| {
            json!({
                "name": t.name(),
                "description": t.description(),
                "parameters": t.parameters_schema()
            })
        }).collect::<Vec<_>>();
        
        // Manual prompt injection for tools
        let tools_prompt = format!(
            "You are an intelligent agent with access to the following tools:\n\n{}\n\nRULES:\n1. To use a tool, you MUST output ONLY a valid JSON object matching the 'tool' and 'args' schema.\n2. Example: {{\"tool\": \"filesystem\", \"args\": {{\"operation\": \"list_dir\", \"path\": \".\"}}}}\n3. If a user asks to perform an action available via tools (like listing files), USE THE TOOL.\n4. Do not apologize or ask for clarification if the request is clear and you have a tool for it.\n5. Output ONLY the JSON for the tool call, no other text.",
            serde_json::to_string_pretty(&tools_desc).unwrap()
        );

        let agent = client.agent(&self.model)
            .preamble(&tools_prompt)
            .build();

        let max_steps = 10;
        let mut steps_count = 0;
        let mut final_response_text = String::new();

        loop {
            if steps_count >= max_steps {
                break;
            }
            steps_count += 1;
            
            // Let's pop the last message to use as the prompt argument.
            let prompt_content = if let Some(last_msg) = self.history.pop() {
                last_msg.content
            } else {
                "continue".to_string() // Fallback
            };
            
            // Put prompt back into history for next iteration/persistence logic
            self.history.push(rig::completion::Message { role: "user".to_string(), content: prompt_content.clone() });

            // Fallback to non-streaming prompt() due to compilation issues with rig-core 0.6.0 streaming
            let response = match agent.prompt(&prompt_content).await {
                Ok(r) => r,
                Err(e) => {
                    println!("Error: {}", e);
                     let _ = Emitter::emit(&window, &format!("session:{}", self.session_id), AgentEvent::Token {
                        content: format!("Error: {}", e),
                    });
                    break;
                }
            };
            
            // Check if it's a tool call
            if let Ok(tool_call) = serde_json::from_str::<Value>(&response) {
                if let (Some(tool_name), Some(args)) = (tool_call["tool"].as_str(), tool_call.get("args")) {
                    // It is a tool call
                    if let Some(tool) = self.tools.iter().find(|t| t.name() == tool_name) {
                        
                        let step_id = Uuid::new_v4().to_string();
                        let step = ExecutionStep {
                            id: step_id.clone(),
                            tool_name: tool_name.to_string(),
                            tool_args: args.clone(),
                            status: "executing".to_string(),
                            result: None,
                            requires_approval: false, // Handled internally by tool now
                            created_at: chrono::Utc::now().to_rfc3339(),
                        };

                        // 2. Execution
                        let _ = Emitter::emit(&window, &format!("session:{}", self.session_id), AgentEvent::StepStarted {
                            job: job.clone(),
                            step: step.clone(),
                        });

                        let _ = Emitter::emit(&window, &format!("session:{}", self.session_id), AgentEvent::Thinking {
                            message: format!("Executing {}...", tool_name),
                        });
                        
                        let ctx = ToolContext {
                            permissions: permission_manager.clone(),
                            window: Some(window.clone()), 
                            session_id: self.session_id.clone(),
                        };

                        // SNAPSHOT START
                        let pre_snapshot = self.snapshot_manager.create_snapshot().ok();
                    
                        let result = tool.execute(args.clone(), &ctx).await.unwrap_or_else(|e| Value::String(format!("Error: {}", e)));
                        
                        // SNAPSHOT END & DIFF
                        if let Some(pre) = pre_snapshot {
                            if let Ok(post) = self.snapshot_manager.create_snapshot() {
                                let diff = self.snapshot_manager.diff(&pre, &post);
                                if !diff.new_files.is_empty() || !diff.modified_files.is_empty() || !diff.deleted_files.is_empty() {
                                     let diff_msg = format!("Workspace Changes: +{:?} *{:?} -{:?}", diff.new_files, diff.modified_files, diff.deleted_files);
                                     let _ = Emitter::emit(&window, &format!("session:{}", self.session_id), AgentEvent::Thinking {
                                        message: diff_msg,
                                    });
                                }
                            }
                        }
                        
                         let _ = Emitter::emit(&window, &format!("session:{}", self.session_id), AgentEvent::StepCompleted {
                            job: job.clone(),
                            step: ExecutionStep {
                                status: "completed".to_string(),
                                result: Some(result.clone().to_string()),
                                ..step
                            },
                        });

                        // Add result to history
                        self.history.push(rig::completion::Message { role: "assistant".to_string(), content: response.to_string() });
                        self.history.push(rig::completion::Message { role: "user".to_string(), content: format!("Tool '{}' result: {}", tool_name, result) });
                        
                        continue; // Loop again
                    }
                }
            }
            
            // Not a tool call implies text response
            // Not a tool call implies text response
            final_response_text = response.clone();
            let mut processor = StreamProcessor::new();
            Self::stream_text(&window, &self.session_id, &response, &mut processor).await;
            
            // Add to history
            self.history.push(rig::completion::Message { role: "assistant".to_string(), content: final_response_text.clone() });
            break;
        }

        // Save to DB
        save_message(&db_pool, "assistant", &final_response_text, &self.session_id);

        let final_msg = if final_response_text.is_empty() {
            "Done".to_string()
        } else {
            final_response_text.clone()
        };

        let _ = Emitter::emit(&window, &format!("session:{}", self.session_id), AgentEvent::JobCompleted {
            job: ExecutionJob { status: "completed".to_string(), ..job },
            message: final_msg,
        });
    }
    
    // Keep helper
    async fn stream_text(window: &tauri::WebviewWindow<R>, session_id: &str, text: &str, processor: &mut StreamProcessor) {
        // Feed text to processor as chunks (simulating token stream for now since we don't have real stream yet)
        // In real impl, this would be inside the loop receiving tokens from LLM.
        // Here we just split by space to simulate.
        let words: Vec<&str> = text.split_inclusive(' ').collect(); // inclusive keeps separators
        let mut tokens = words; 
        if tokens.is_empty() && !text.is_empty() { tokens = vec![text]; }

        for (_i, token) in tokens.iter().enumerate() {
            let chunks = processor.process(token);
            for chunk in chunks {
                match chunk {
                    StreamChunk::Text(content) => {
                         let _ = Emitter::emit(window, &format!("session:{}", session_id), AgentEvent::Token { content });
                    },
                    StreamChunk::Thinking(content) => {
                         let _ = Emitter::emit(window, &format!("session:{}", session_id), AgentEvent::Thinking { message: content });
                    }
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
        }
    }
}

fn save_message(db_pool: &DbPool, role: &str, content: &str, session_id: &str) {
    use crate::schema::messages;
    if let Ok(mut conn) = db_pool.get() {
        let msg = crate::models::NewMessage {
            id: Uuid::new_v4().to_string(),
            role: role.to_string(),
            content: content.to_string(),
            session_id: session_id.to_string(),
            metadata_json: None,
            tokens: None,
        };
        let _ = diesel::insert_into(messages::table).values(&msg).execute(&mut conn);
    }
}

// Updated start_chat_task
// Updated start_chat_task
pub fn start_chat_task<R: Runtime>(
    mut agent_loop: AgentLoop<R>,
    message: String,
    session_id: String,
    window: tauri::WebviewWindow<R>,
    pending_approvals: Arc<dashmap::DashMap<String, oneshot::Sender<bool>>>,
    permission_manager: Arc<PermissionManager>,
    db_pool: DbPool,
) {
    let job_id = Uuid::new_v4().to_string();
    agent_loop.session_id = session_id;
    
    tauri::async_runtime::spawn(async move {
        agent_loop.run(message, window, job_id, pending_approvals, permission_manager, db_pool).await;
    });
}
