pub mod processor;
pub mod optimizations;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod workflow_tests;

use rig::completion::Prompt;
use rig::completion::Chat;
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
use jsonschema::JSONSchema;
use optimizations::{
    truncate_tool_result,
    truncate_message_content,
    optimize_history_by_tokens,
    MAX_HISTORY_TOKENS,
};


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
        let truncated_user_message = truncate_message_content(&user_message, "user");
        self.history.push(rig::completion::Message { role: "user".to_string(), content: truncated_user_message });

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
            "You are an intelligent agent with access to the following tools:\n\n{}\n\nRULES:\n1. To use a tool, you MUST output ONLY a valid JSON object matching the 'tool' and 'args' schema.\n2. Example: {{\"tool\": \"filesystem\", \"args\": {{\"operation\": \"list_dir\", \"path\": \".\"}}}}\n3. If a user asks to perform an action available via tools (like listing files), USE THE TOOL. Do NOT describe what you will do, just do it.\n4. Do not apologize or ask for clarification if the request is clear and you have a tool for it.\n5. Output ONLY the JSON for the tool call, no other text.\n6. For multi-step tasks, execute the first step immediately. Do NOT stop until all requested steps are completed.\n7. ONLY if the task is fully completed or you cannot proceed, then output plain text to answer.",
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
            let prompt_msg = if let Some(last_msg) = self.history.pop() {
                last_msg
            } else {
                rig::completion::Message { role: "user".to_string(), content: "continue".to_string() }
            };
            
            let current_history = self.history.clone();

            // Put prompt back into history for next iteration/persistence logic
            self.history.push(prompt_msg.clone());

            // Use chat() to include history context
            let response = match agent.chat(&prompt_msg.content, current_history).await {
                Ok(r) => r.to_string(),
                Err(e) => {
                    println!("Error: {}", e);
                     let _ = Emitter::emit(&window, &format!("session:{}", self.session_id), AgentEvent::Token {
                        content: format!("Error: {}", e),
                    });
                    break;
                }
            };
            
            // Attempt to parse tool call
            // Attempt to parse tool call
            let tool_call_json = extract_tool_call(&response);

            // Check if it's a tool call
            if let Some(tool_call) = tool_call_json {
                if let (Some(tool_name), Some(args)) = (tool_call["tool"].as_str(), tool_call.get("args")) {
                    // It is a tool call
                    if let Some(tool) = self.tools.iter().find(|t| t.name() == tool_name) {
                        // Persist the Assistant's Tool Call
                        let truncated_response = truncate_message_content(&response, "assistant");
                        // For assistant tool calls, we could store metadata, but usually the text is enough or we parse it.
                        // However, to be consistent, let's just pass None or the args if we wanted.
                        // For now, let's keep it simple for the assistant side as the UI parses the text fine for "Thinking".
                        save_message(&db_pool, "assistant", &truncated_response, &self.session_id, None);
                        self.history.push(rig::completion::Message { role: "assistant".to_string(), content: truncated_response });

                        // VALIDATION START
                        let schema_json = tool.parameters_schema();
                        let compiled_schema = JSONSchema::compile(&schema_json).unwrap_or_else(|e| panic!("Invalid schema for tool {}: {}", tool_name, e));
                        
                        if let Err(errors) = compiled_schema.validate(args) {
                            let error_msg = errors.map(|e| e.to_string()).collect::<Vec<String>>().join(", ");
                            let fail_msg = format!("Tool argument validation failed (schema): {}", error_msg);

                            let step_id = Uuid::new_v4().to_string();
                            let step = ExecutionStep {
                                id: step_id.clone(),
                                tool_name: tool_name.to_string(),
                                tool_args: args.clone(),
                                status: "failed".to_string(),
                                result: Some(fail_msg.clone()),
                                requires_approval: false,
                                created_at: chrono::Utc::now().to_rfc3339(),
                            };

                            let _ = Emitter::emit(&window, &format!("session:{}", self.session_id), AgentEvent::StepStarted {
                                job: job.clone(),
                                step: step.clone(),
                            });
                             let _ = Emitter::emit(&window, &format!("session:{}", self.session_id), AgentEvent::StepCompleted {
                                job: job.clone(),
                                step: step.clone(),
                            });

                             // Add failure to history and DB
                            let fail_msg_full = format!("Tool '{}' validation failed: {}", tool_name, fail_msg);
                            let truncated_fail_msg = truncate_message_content(&fail_msg_full, "user");
                            save_message(&db_pool, "tool", &truncated_fail_msg, &self.session_id, Some(args.to_string()));
                            self.history.push(rig::completion::Message { role: "user".to_string(), content: truncated_fail_msg });

                            continue;
                        }
                        
                        // 2. Logic Validation
                        if let Err(e) = tool.validate_args(args).await {
                             let fail_msg = format!("Tool argument validation failed (logic): {}", e);
                             let step_id = Uuid::new_v4().to_string();
                             let step = ExecutionStep {
                                id: step_id.clone(),
                                tool_name: tool_name.to_string(),
                                tool_args: args.clone(),
                                status: "failed".to_string(),
                                result: Some(fail_msg.clone()),
                                requires_approval: false,
                                created_at: chrono::Utc::now().to_rfc3339(),
                            };
                             let _ = Emitter::emit(&window, &format!("session:{}", self.session_id), AgentEvent::StepStarted { job: job.clone(), step: step.clone() });
                             
                             // Add failure to history and DB
                             let fail_msg_full = format!("Tool '{}' validation failed: {}", tool_name, fail_msg);
                             let truncated_fail_msg = truncate_message_content(&fail_msg_full, "user");
                             save_message(&db_pool, "tool", &truncated_fail_msg, &self.session_id, Some(args.to_string()));
                             self.history.push(rig::completion::Message { role: "user".to_string(), content: truncated_fail_msg });
                             continue;
                        }

                        // VALIDATION END
                        
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
                    
                         let execution_result = tool.execute(args.clone(), &ctx).await.unwrap_or_else(|e| Value::String(format!("Error: {}", e)));
                        
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

                        // 3. Verification
                        let success = tool.verify_result(&execution_result);
                        let status = if success { "completed" } else { "failed" };
                        
                        // 4. Summarization
                        let mut final_result = execution_result.to_string();

                        // Smart truncation (Safety to prevent token overflow)
                        final_result = truncate_tool_result(tool_name, &final_result);

                        if success && tool.needs_summarization(&args, &execution_result) {
                             let _ = Emitter::emit(&window, &format!("session:{}", self.session_id), AgentEvent::Thinking {
                                message: format!("Summarizing output from {}...", tool_name),
                            });
                            
                            // Simple summarization using the same client/model
                            let summary_agent = client.agent(&self.model)
                                .preamble("You are a helpful assistant. Summarize the following tool output concisely.")
                                .build();
                            
                            if let Ok(summary) = summary_agent.prompt(&final_result).await {
                                final_result = format!("(Summary) {}", summary);
                            }
                        }

                        let _ = Emitter::emit(&window, &format!("session:{}", self.session_id), AgentEvent::StepCompleted {
                            job: job.clone(),
                            step: ExecutionStep {
                                status: status.to_string(),
                                result: Some(final_result.clone()),
                                ..step
                            },
                        });

                        // Add result to history and DB
                        let tool_result_msg = format!("Tool '{}' result: {}", tool_name, final_result);
                        let truncated_tool_result = truncate_message_content(&tool_result_msg, "user");
                        
                        // IMPORTANT: Save as 'tool' role with args as metadata
                        save_message(&db_pool, "tool", &truncated_tool_result, &self.session_id, Some(args.to_string()));
                        
                        self.history.push(rig::completion::Message { role: "user".to_string(), content: truncated_tool_result });

                        // Optimize history to prevent context overflow
                        optimize_history_by_tokens(&mut self.history, MAX_HISTORY_TOKENS);

                        continue; // Loop again
                    }
                }
            }
            
            // Not a tool call implies text response
            final_response_text = response.clone();
            let mut processor = StreamProcessor::new();
            Self::stream_text(&window, &self.session_id, &response, &mut processor).await;

            // Add to history with truncation
            let truncated_final_response = truncate_message_content(&final_response_text, "assistant");
            self.history.push(rig::completion::Message { role: "assistant".to_string(), content: truncated_final_response });
            break;
        }

        // Save to DB
        save_message(&db_pool, "assistant", &final_response_text, &self.session_id, None);

        let final_msg = if final_response_text.is_empty() {
            "Task completed successfully. All requested actions have been executed.".to_string()
        } else {
            final_response_text.clone()
        };

        // Auto-generate title if this is the first turn
        if self.history.len() <= 2 {
            let session_id_clone = self.session_id.clone();
            let model_clone = self.model.clone();
            let db_pool_clone = db_pool.clone();
            let window_clone = window.clone();
            let user_query = user_message.clone(); // Capture original query

            tauri::async_runtime::spawn(async move {
                let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_default();
                if !api_key.is_empty() {
                    let client = openai::Client::new(&api_key);
                    let title_agent = client.agent(&model_clone)
                        .preamble("You are a helpful assistant. Generate a very concise title (3-5 words) for a chat session based on the user's first message. Do not use quotes. Do not say 'Title:'. Just the title.")
                        .build();

                    if let Ok(title) = title_agent.prompt(&user_query).await {
                         let clean_title = title.trim().trim_matches('"').to_string();
                         if !clean_title.is_empty() {
                             // Update DB
                             use crate::schema::sessions;
                             if let Ok(mut conn) = db_pool_clone.get() {
                                 let _ = diesel::update(sessions::table.filter(sessions::id.eq(&session_id_clone)))
                                     .set(sessions::title.eq(&clean_title))
                                     .execute(&mut conn);
                                 
                                     // Emit update event to refresh sidebar
                                     let _ = Emitter::emit(&window_clone, "sessions_updated", ());
                             }
                         }
                    }
                }
            });
        }

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

fn save_message(db_pool: &DbPool, role: &str, content: &str, session_id: &str, metadata_json: Option<String>) {
    use crate::schema::messages;
    if let Ok(mut conn) = db_pool.get() {
        let msg = crate::models::NewMessage {
            id: Uuid::new_v4().to_string(),
            role: role.to_string(),
            content: content.to_string(),
            session_id: session_id.to_string(),
            metadata_json: metadata_json,
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

fn extract_tool_call(response: &str) -> Option<Value> {
    // 1. Try pure JSON
    if let Ok(json) = serde_json::from_str::<Value>(response) {
        if json.is_object() {
            return Some(json);
        }
    }

    // 2. Try identifying JSON in markdown code blocks like ```json ... ```
    // Note: This is a simple heurisitic, rigorous parsing might need regex
    // But find('{') and rfind('}') usually works well enough if there is only one JSON object.

    if let Some(start) = response.find('{') {
        if let Some(end) = response.rfind('}') {
            if start <= end {
                // Slicing in Rust is [start..end) (exclusive), so strict inequality for empty {} is fine
                // But to include the '}', we need end + 1.
                // We must be careful that end + 1 is a valid char boundary.
                // Since '}' is ASCII (1 byte), end + 1 is safe if response ends with '}'.
                // However, rfind returns the index of the start of key char.
                // '}' is 1 byte, so end + 1 is the correct limit.
                
                let limit = end + 1;
                if limit <= response.len() {
                    let potential_json = &response[start..limit];
                    if let Ok(json) = serde_json::from_str::<Value>(potential_json) {
                        if json.is_object() {
                            return Some(json);
                        }
                    }
                }
            }
        }
    }
    
    None
}

#[cfg(test)]
mod parser_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_extract_pure_json() {
        let input = r#"{"tool": "test", "args": {}}"#;
        let result = extract_tool_call(input);
        assert!(result.is_some());
        assert_eq!(result.unwrap()["tool"], "test");
    }

    #[test]
    fn test_extract_embedded_json() {
        let input = r#"I will run this tool: {"tool": "test", "args": {}}"#;
        let result = extract_tool_call(input);
        assert!(result.is_some());
        assert_eq!(result.unwrap()["tool"], "test");
    }

    #[test]
    fn test_extract_markdown_json() {
        let input = r#"Here is the code:
```json
{
    "tool": "test", 
    "args": {}
}
```
"#;
        let result = extract_tool_call(input);
        assert!(result.is_some());
        assert_eq!(result.unwrap()["tool"], "test");
    }

    #[test]
    fn test_extract_suffix_text() {
        let input = r#"{"tool": "test", "args": {}} is the tool I used."#;
        let result = extract_tool_call(input);
        assert!(result.is_some());
        assert_eq!(result.unwrap()["tool"], "test");
    }

    #[test]
    fn test_extract_invalid_json() {
        let input = r#"This is just text with { brackets } but no valid json."#;
        let result = extract_tool_call(input);
        assert!(result.is_none());
    }
}
