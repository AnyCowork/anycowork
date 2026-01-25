pub mod optimizations;
pub mod processor;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod workflow_tests;

use crate::database::DbPool;
use crate::events::{AgentEvent, ExecutionJob, ExecutionStep};
use crate::models::Agent as DbAgent;
use crate::permissions::PermissionManager;
use crate::tools::{
    bash::BashTool, filesystem::FilesystemTool, search::SearchTool, Tool, ToolContext,
};
use diesel::prelude::*;
use jsonschema::JSONSchema;
use log::error;
use optimizations::{
    create_assistant_message, create_user_message, get_message_content, optimize_history_by_tokens,
    truncate_message_content, truncate_tool_result, MAX_HISTORY_TOKENS,
};
use processor::{StreamChunk, StreamProcessor};
use rig::agent::Agent;
use rig::client::CompletionClient;
use rig::client::ProviderClient;
use rig::completion::Chat;
use rig::completion::CompletionModel;
use rig::completion::Prompt;
use rig::providers::anthropic;
use rig::providers::gemini;
use rig::providers::openai;
use serde_json::{json, Value};
use std::sync::Arc;
use tauri::Emitter;
use tokio::sync::oneshot;
use uuid::Uuid;

use tauri::Runtime;

pub struct AgentLoop<R: Runtime> {
    pub agent_id: String,
    pub session_id: String,
    pub model: String,
    pub provider: String,
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
            provider: agent_db.ai_provider.clone(),
            history: vec![],
            tools,
            snapshot_manager: crate::snapshots::SnapshotManager::new(
                std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")),
            ),
        }
    }

    pub async fn run(
        &mut self,
        user_message: String,
        window: tauri::WebviewWindow<R>,
        job_id: String,
        _pending_approvals: Arc<dashmap::DashMap<String, oneshot::Sender<bool>>>,
        permission_manager: Arc<PermissionManager>,
        db_pool: DbPool,
    ) {
        // Initialize job
        let job = ExecutionJob {
            id: job_id.clone(),
            session_id: self.session_id.clone(),
            status: "running".to_string(),
            query: user_message.clone(),
            steps: vec![],
            current_step_index: 0,
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        let _ = Emitter::emit(
            &window,
            &format!("session:{}", self.session_id),
            AgentEvent::JobStarted { job: job.clone() },
        );

        // 1. Add User Message to History
        let truncated_user_message = truncate_message_content(&user_message, "user");
        self.history
            .push(create_user_message(truncated_user_message));

        // Prepare tool definitions
        let tools_desc = self
            .tools
            .iter()
            .map(|t| {
                json!({
                    "name": t.name(),
                    "description": t.description(),
                    "parameters": t.parameters_schema()
                })
            })
            .collect::<Vec<_>>();

        // Load tool use prompt template
        // Using runtime read as per user request for data separation
        let template_str = std::fs::read_to_string("src-tauri/prompts/tool_use_system.j2")
            // Fallback for dev/build environments where file might not be relative to CWD
            .unwrap_or_else(|_| include_str!("../../prompts/tool_use_system.j2").to_string());

        let mut env = minijinja::Environment::new();
        env.add_template("tool_use", &template_str).unwrap(); // Panic if template invalid (should only happen in dev)

        let tools_json = serde_json::to_string_pretty(&tools_desc).unwrap();
        let tmpl = env.get_template("tool_use").unwrap();

        // Render prompt (Plan context can be passed here if extended)
        let tools_prompt = tmpl
            .render(minijinja::context! {
                tools_desc => tools_json,
                plan => Value::Null,
                current_task => Value::Null
            })
            .unwrap();

        // Select and Build Agent based on Provider
        // Dispatching to a common handler or using Box<dyn Chat> would be ideal,
        // but Rig agents are typed by their completion model.
        // We will match and dispatch.

        match self.provider.as_str() {
            "openai" => {
                let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_default();
                if api_key.is_empty() {
                    self.send_error(&window, "Error: OPENAI_API_KEY not set", &job);
                    return;
                }
                let client = openai::Client::from_env();
                let agent = client.agent(&self.model).preamble(&tools_prompt).build();
                self.run_loop(
                    agent,
                    &window,
                    &job,
                    permission_manager,
                    &db_pool,
                    user_message,
                )
                .await;
            }
            "gemini" => {
                let api_key = std::env::var("GEMINI_API_KEY").unwrap_or_default();
                if api_key.is_empty() {
                    self.send_error(&window, "Error: GEMINI_API_KEY not set", &job);
                    return;
                }
                let client = gemini::Client::from_env();
                let agent = client.agent(&self.model).preamble(&tools_prompt).build();
                self.run_loop(
                    agent,
                    &window,
                    &job,
                    permission_manager,
                    &db_pool,
                    user_message,
                )
                .await;
            }
            "anthropic" => {
                let api_key = std::env::var("ANTHROPIC_API_KEY").unwrap_or_default();
                if api_key.is_empty() {
                    self.send_error(&window, "Error: ANTHROPIC_API_KEY not set", &job);
                    return;
                }
                let client = anthropic::Client::from_env();
                let agent = client.agent(&self.model).preamble(&tools_prompt).build();
                self.run_loop(
                    agent,
                    &window,
                    &job,
                    permission_manager,
                    &db_pool,
                    user_message,
                )
                .await;
            }
            _ => {
                self.send_error(
                    &window,
                    &format!("Error: Unsupported provider '{}'", self.provider),
                    &job,
                );
            }
        }
    }

    fn send_error(&self, window: &tauri::WebviewWindow<R>, msg: &str, job: &ExecutionJob) {
        let _ = Emitter::emit(
            window,
            &format!("session:{}", self.session_id),
            AgentEvent::Token {
                content: msg.to_string(),
            },
        );
        let _ = Emitter::emit(
            window,
            &format!("session:{}", self.session_id),
            AgentEvent::JobCompleted {
                job: ExecutionJob {
                    status: "failed".to_string(),
                    ..job.clone()
                },
                message: msg.to_string(),
            },
        );
    }

    // Generic run loop using the Agent struct
    pub async fn run_loop<M: CompletionModel>(
        &mut self,
        agent: Agent<M>,
        window: &tauri::WebviewWindow<R>,
        job: &ExecutionJob,
        permission_manager: Arc<PermissionManager>,
        db_pool: &DbPool,
        user_message: String,
    ) {
        let max_steps = 10;
        let mut steps_count = 0;
        let mut final_response_text = String::new();

        loop {
            if steps_count >= max_steps {
                break;
            }
            steps_count += 1;

            let prompt_msg = if let Some(last_msg) = self.history.pop() {
                last_msg
            } else {
                create_user_message("continue".to_string())
            };

            let current_history = self.history.clone();

            // Put prompt back into history for next iteration/persistence logic
            self.history.push(prompt_msg.clone());

            // Use chat() to include history context
            // We implement a retry loop here to handle transient provider errors
            let mut chat_attempts = 0;
            let max_chat_attempts = 3;
            let response = loop {
                chat_attempts += 1;
                match agent
                    .chat(&get_message_content(&prompt_msg), current_history.clone())
                    .await
                {
                    Ok(r) => {
                        break r.to_string();
                    }
                    Err(e) => {
                        error!("Agent chat attempt {} failed: {}", chat_attempts, e);
                        if chat_attempts >= max_chat_attempts {
                            let _ = Emitter::emit(
                                window,
                                &format!("session:{}", self.session_id),
                                AgentEvent::Token {
                                    content: format!(
                                        "Error: {} after {} attempts.",
                                        e, max_chat_attempts
                                    ),
                                },
                            );
                            return; // Fatal error
                        }

                        let wait_ms = 1000 * (2u64.pow(chat_attempts as u32 - 1));
                        let retry_msg = format!(
                            "\n⚠️ Connection issue. Retrying in {}s... (Attempt {}/{})\n",
                            wait_ms as f64 / 1000.0,
                            chat_attempts + 1,
                            max_chat_attempts
                        );
                        let _ = Emitter::emit(
                            window,
                            &format!("session:{}", self.session_id),
                            AgentEvent::Thinking { message: retry_msg },
                        );
                        tokio::time::sleep(tokio::time::Duration::from_millis(wait_ms)).await;
                    }
                }
            };

            // Post-process response to show thinking if it contains text before tool
            if let Some(json_start) = response.find('{') {
                if response[json_start..].contains("\"tool\"") {
                    let thinking_text = response[..json_start].trim();
                    if !thinking_text.is_empty() {
                        let _ = Emitter::emit(
                            window,
                            &format!("session:{}", self.session_id),
                            AgentEvent::Thinking {
                                message: thinking_text.to_string(),
                            },
                        );
                    }
                }
            }

            // Attempt to parse tool calls
            let tool_calls = extract_tool_calls(&response);

            // Filter to find valid tool calls first to decide if we should enter tool execution mode
            let valid_calls: Vec<(String, Value, &Box<dyn Tool<R>>)> = tool_calls
                .into_iter()
                .filter_map(|tc| {
                    let tool_name = tc.get("tool")?.as_str()?.to_string();
                    let args = tc.get("args")?.clone();
                    let tool = self.tools.iter().find(|t| t.name() == tool_name)?;
                    Some((tool_name, args, tool))
                })
                .collect();

            if !valid_calls.is_empty() {
                // Persist the Assistant's Response (with all tool calls) ONCE
                let truncated_response = truncate_message_content(&response, "assistant");
                save_message(db_pool, "assistant", &response, &self.session_id, None);
                self.history
                    .push(create_assistant_message(truncated_response));

                for (tool_name, args, tool) in valid_calls {
                    // VALIDATION START
                    let schema_json = tool.parameters_schema();
                    let compiled_schema = JSONSchema::compile(&schema_json)
                        .unwrap_or_else(|e| panic!("Invalid schema for tool {}: {}", tool_name, e));

                    if let Err(errors) = compiled_schema.validate(&args) {
                        let error_msg = errors
                            .map(|e| e.to_string())
                            .collect::<Vec<String>>()
                            .join(", ");
                        let fail_msg =
                            format!("Tool argument validation failed (schema): {}", error_msg);

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

                        let _ = Emitter::emit(
                            window,
                            &format!("session:{}", self.session_id),
                            AgentEvent::StepStarted {
                                job: job.clone(),
                                step: step.clone(),
                            },
                        );
                        let _ = Emitter::emit(
                            window,
                            &format!("session:{}", self.session_id),
                            AgentEvent::StepCompleted {
                                job: job.clone(),
                                step: step.clone(),
                            },
                        );

                        // Add failure to history and DB
                        let fail_msg_full =
                            format!("Tool '{}' validation failed: {}", tool_name, fail_msg);
                        let truncated_fail_msg = truncate_message_content(&fail_msg_full, "user");
                        save_message(
                            db_pool,
                            "tool",
                            &fail_msg_full,
                            &self.session_id,
                            Some(args.to_string()),
                        );
                        self.history.push(create_user_message(truncated_fail_msg));

                        continue;
                    }

                    // 2. Logic Validation
                    if let Err(e) = tool.validate_args(&args).await {
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
                        let _ = Emitter::emit(
                            window,
                            &format!("session:{}", self.session_id),
                            AgentEvent::StepStarted {
                                job: job.clone(),
                                step: step.clone(),
                            },
                        );

                        // Add failure to history and DB
                        let fail_msg_full =
                            format!("Tool '{}' validation failed: {}", tool_name, fail_msg);
                        let truncated_fail_msg = truncate_message_content(&fail_msg_full, "user");
                        save_message(
                            db_pool,
                            "tool",
                            &fail_msg_full,
                            &self.session_id,
                            Some(args.to_string()),
                        );
                        self.history.push(create_user_message(truncated_fail_msg));
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
                    let _ = Emitter::emit(
                        window,
                        &format!("session:{}", self.session_id),
                        AgentEvent::StepStarted {
                            job: job.clone(),
                            step: step.clone(),
                        },
                    );

                    let _ = Emitter::emit(
                        window,
                        &format!("session:{}", self.session_id),
                        AgentEvent::Thinking {
                            message: format!("Executing {}...", tool_name),
                        },
                    );

                    let ctx = ToolContext {
                        permissions: permission_manager.clone(),
                        window: Some(window.clone()),
                        session_id: self.session_id.clone(),
                    };

                    // SNAPSHOT START
                    let pre_snapshot = self.snapshot_manager.create_snapshot().ok();

                    let execution_result = tool
                        .execute(args.clone(), &ctx)
                        .await
                        .unwrap_or_else(|e| Value::String(format!("Error: {}", e)));

                    // SNAPSHOT END & DIFF
                    if let Some(pre) = pre_snapshot {
                        if let Ok(post) = self.snapshot_manager.create_snapshot() {
                            let diff = self.snapshot_manager.diff(&pre, &post);
                            if !diff.new_files.is_empty()
                                || !diff.modified_files.is_empty()
                                || !diff.deleted_files.is_empty()
                            {
                                let diff_msg = format!(
                                    "Workspace Changes: +{:?} *{:?} -{:?}",
                                    diff.new_files, diff.modified_files, diff.deleted_files
                                );
                                let _ = Emitter::emit(
                                    window,
                                    &format!("session:{}", self.session_id),
                                    AgentEvent::Thinking { message: diff_msg },
                                );
                            }
                        }
                    }

                    // 3. Verification
                    let success = tool.verify_result(&execution_result);
                    let status = if success { "completed" } else { "failed" };

                    // 4. Summarization
                    let mut final_result = execution_result.to_string();

                    // Smart truncation (Safety to prevent token overflow)
                    final_result = truncate_tool_result(&tool_name, &final_result);

                    let _ = Emitter::emit(
                        window,
                        &format!("session:{}", self.session_id),
                        AgentEvent::StepCompleted {
                            job: job.clone(),
                            step: ExecutionStep {
                                status: status.to_string(),
                                result: Some(final_result.clone()),
                                ..step
                            },
                        },
                    );

                    // Add result to history and DB
                    let tool_result_msg = format!("Tool '{}' result: {}", tool_name, final_result);
                    let truncated_tool_result = truncate_message_content(&tool_result_msg, "user");

                    // IMPORTANT: Save as 'tool' role with args as metadata
                    save_message(
                        db_pool,
                        "tool",
                        &tool_result_msg,
                        &self.session_id,
                        Some(args.to_string()),
                    );

                    self.history
                        .push(create_user_message(truncated_tool_result));
                }

                // Optimize history to prevent context overflow
                optimize_history_by_tokens(&mut self.history, MAX_HISTORY_TOKENS);

                continue; // Loop again
            }

            // Not a tool call implies text response
            final_response_text = response.clone();
            let mut processor = StreamProcessor::new();
            Self::stream_text(window, &self.session_id, &response, &mut processor).await;

            // Add to history with truncation
            let truncated_final_response =
                truncate_message_content(&final_response_text, "assistant");
            self.history
                .push(create_assistant_message(truncated_final_response));
            break;
        }

        // Save to DB
        save_message(
            db_pool,
            "assistant",
            &final_response_text,
            &self.session_id,
            None,
        );

        let final_msg = if final_response_text.is_empty() {
            "Task completed successfully. All requested actions have been executed.".to_string()
        } else {
            final_response_text.clone()
        };

        // Auto-generate title if this is the first turn
        // Auto-generate title logic
        // 1. New Chat (Default) - handled by UI if title is Null
        // 2. First Message (History < 3) - Set to first message
        // 3. LLM Generated (History >= 3) - Generate summary

        let history_len = self.history.len();

        // We only check DB if we are in the fallback stage (< 3) to strictly avoid overwriting
        // OR if we are in the generation window (>= 3)

        if history_len < 3 {
            // FALLBACK STAGE: Set title to User Message if it's currently default
            use crate::schema::sessions;
            let should_fallback = if let Ok(mut conn) = db_pool.get() {
                let current_title: Option<String> = sessions::table
                    .filter(sessions::id.eq(&self.session_id))
                    .select(sessions::title)
                    .first(&mut conn)
                    .unwrap_or(None);

                // If title is None, empty, or "New Chat" -> Update it
                match current_title {
                    Some(t) => t.is_empty() || t == "New Chat",
                    None => true,
                }
            } else {
                false
            };

            if should_fallback {
                let fallback_title = user_message.chars().take(30).collect::<String>();
                if let Ok(mut conn) = db_pool.get() {
                    let _ =
                        diesel::update(sessions::table.filter(sessions::id.eq(&self.session_id)))
                            .set(sessions::title.eq(&fallback_title))
                            .execute(&mut conn);
                    let _ = Emitter::emit(window, "sessions_updated", ());
                }
            }
        } else if (3..=10).contains(&history_len) {
            // LLM GENERATION STAGE: Generate summary
            // We do this a few times (Turn 2, 3, 4) to refine the title as context grows, then stop to stay stable.

            let session_id_clone = self.session_id.clone();
            let model_clone = self.model.clone();
            let provider_clone = self.provider.clone();
            let db_pool_clone = db_pool.clone();
            let window_clone = window.clone();
            let user_query = user_message.clone(); // Capture original query

            tauri::async_runtime::spawn(async move {
                let preamble = "You are a helpful assistant. Generate a very concise title (3-5 words) for a chat session based on the interaction so far. Do not use quotes. Do not say 'Title:'. Just the title.";

                let title_result = match provider_clone.as_str() {
                    "openai" => {
                        let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_default();
                        if !api_key.is_empty() {
                            let client = openai::Client::from_env();
                            let agent = client.agent(&model_clone).preamble(preamble).build();
                            agent.prompt(&user_query).await.ok() // Using user_query as prompt trigger, ideally send history but simple prompt often works for title
                        } else {
                            None
                        }
                    }
                    "gemini" => {
                        let api_key = std::env::var("GEMINI_API_KEY").unwrap_or_default();
                        if !api_key.is_empty() {
                            let client = gemini::Client::from_env();
                            // Use Flash for titles if possible for speed, otherwise fallback to current model
                            let title_model = if model_clone.contains("flash") {
                                model_clone.clone()
                            } else {
                                "gemini-1.5-flash".to_string()
                            };
                            let agent = client.agent(&title_model).preamble(preamble).build();
                            agent.prompt(&user_query).await.ok()
                        } else {
                            None
                        }
                    }
                    "anthropic" => {
                        let api_key = std::env::var("ANTHROPIC_API_KEY").unwrap_or_default();
                        if !api_key.is_empty() {
                            let client = anthropic::Client::from_env();
                            // Use Haiku for titles for speed
                            let title_model = "claude-3-haiku-20240307";
                            let agent = client.agent(title_model).preamble(preamble).build();
                            agent.prompt(&user_query).await.ok()
                        } else {
                            None
                        }
                    }
                    _ => None,
                };

                if let Some(title) = title_result {
                    let clean_title = title.trim().trim_matches('"').to_string();
                    if !clean_title.is_empty() {
                        // Update DB
                        use crate::schema::sessions;
                        if let Ok(mut conn) = db_pool_clone.get() {
                            let _ = diesel::update(
                                sessions::table.filter(sessions::id.eq(&session_id_clone)),
                            )
                            .set(sessions::title.eq(&clean_title))
                            .execute(&mut conn);

                            // Emit update event to refresh sidebar
                            let _ = Emitter::emit(&window_clone, "sessions_updated", ());
                        }
                    }
                }
            });
        }

        let _ = Emitter::emit(
            window,
            &format!("session:{}", self.session_id),
            AgentEvent::JobCompleted {
                job: ExecutionJob {
                    status: "completed".to_string(),
                    ..job.clone()
                },
                message: final_msg,
            },
        );
    }

    // Keep helper
    async fn stream_text(
        window: &tauri::WebviewWindow<R>,
        session_id: &str,
        text: &str,
        processor: &mut StreamProcessor,
    ) {
        // Feed text to processor as chunks (simulating token stream for now since we don't have real stream yet)
        // In real impl, this would be inside the loop receiving tokens from LLM.
        // Here we just split by space to simulate.
        let words: Vec<&str> = text.split_inclusive(' ').collect(); // inclusive keeps separators
        let mut tokens = words;
        if tokens.is_empty() && !text.is_empty() {
            tokens = vec![text];
        }

        for token in tokens.iter() {
            let chunks = processor.process(token);
            for chunk in chunks {
                match chunk {
                    StreamChunk::Text(content) => {
                        let _ = Emitter::emit(
                            window,
                            &format!("session:{}", session_id),
                            AgentEvent::Token { content },
                        );
                    }
                    StreamChunk::Thinking(content) => {
                        let _ = Emitter::emit(
                            window,
                            &format!("session:{}", session_id),
                            AgentEvent::Thinking { message: content },
                        );
                    }
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
        }
    }
}

fn save_message(
    db_pool: &DbPool,
    role: &str,
    content: &str,
    session_id: &str,
    metadata_json: Option<String>,
) {
    use crate::schema::messages;
    if let Ok(mut conn) = db_pool.get() {
        let msg = crate::models::NewMessage {
            id: Uuid::new_v4().to_string(),
            role: role.to_string(),
            content: content.to_string(),
            session_id: session_id.to_string(),
            metadata_json,
            tokens: None,
        };
        let _ = diesel::insert_into(messages::table)
            .values(&msg)
            .execute(&mut conn);
    }
}

// Updated start_chat_task
// Updated start_chat_task
pub mod coordinator;
pub mod planner;

use coordinator::Coordinator;

// Updated start_chat_task
#[allow(clippy::too_many_arguments)]
pub fn start_chat_task<R: Runtime>(
    // mut agent_loop: AgentLoop<R>, // No longer need AgentLoop passed in, or we wrap it in Coordinator
    // Actually start_chat_task signature shouldn't change too much if we want to avoid breaking callers.
    // Callers pass AgentLoop. We can extract info from it or just change caller.
    // Let's change this function signature slightly or extract fields.
    // Existing callers: `src-tauri/src/lib.rs` (likely).
    // Let's keep signature similar but expect slightly different args if needed.
    // But wait, AgentLoop::new is async. Callers usually do: `let loop = AgentLoop::new(...)`.
    // Let's take the necessary components to build Coordinator.

    // Changing signature:
    agent_db: DbAgent, // Need DB agent config to build Coordinator/Planner
    message: String,
    session_id: String,
    window: tauri::WebviewWindow<R>,
    pending_approvals: Arc<dashmap::DashMap<String, oneshot::Sender<bool>>>,
    permission_manager: Arc<PermissionManager>,
    db_pool: DbPool,
    mode: String,
) {
    let coordinator = Coordinator::new(
        session_id,
        agent_db,
        window,
        db_pool,
        permission_manager,
        pending_approvals,
        mode,
    );

    tauri::async_runtime::spawn(async move {
        coordinator.run(message).await;
    });
}

fn extract_tool_calls(response: &str) -> Vec<Value> {
    let mut calls = Vec::new();

    // 1. Try pure JSON first
    if let Ok(json) = serde_json::from_str::<Value>(response) {
        if json.is_object() && json.get("tool").is_some() {
            return vec![json];
        }
        // If it's an array of valid tool calls (rare but possible)
        if let Some(arr) = json.as_array() {
            if arr.iter().all(|v| v.is_object() && v.get("tool").is_some()) {
                return arr.clone();
            }
        }
    }

    // 2. Scan for JSON objects logic
    let chars: Vec<char> = response.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        if chars[i] == '{' {
            let mut balance = 1;
            let mut j = i + 1;
            let mut in_string = false;
            let mut escape = false;

            while j < len {
                let c = chars[j];

                if escape {
                    escape = false;
                } else if c == '\\' {
                    escape = true;
                } else if c == '"' {
                    in_string = !in_string;
                } else if !in_string {
                    if c == '{' {
                        balance += 1;
                    } else if c == '}' {
                        balance -= 1;
                        if balance == 0 {
                            // Found a balanced block [i ..= j]
                            let candidate: String = chars[i..=j].iter().collect();
                            if let Ok(json) = serde_json::from_str::<Value>(&candidate) {
                                if json.is_object() && json.get("tool").is_some() {
                                    calls.push(json);
                                    // Advance i to j to avoid nested parsing or re-parsing
                                    i = j;
                                    break;
                                }
                            }
                            // If not valid JSON or not tool, just break to continue scanning from i+1?
                            // No, if it was balanced but invalid, we treat it as text.
                            // Actually, if we found a balanced block but it wasn't a tool, we should probably preserve i position?
                            // But `break` here breaks the inner loop.
                            break;
                        }
                    }
                }
                j += 1;
            }
        }
        i += 1;
    }

    calls
}

#[cfg(test)]
mod parser_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_extract_pure_json() {
        let input = r#"{"tool": "test", "args": {}}"#;
        let result = extract_tool_calls(input);
        assert!(!result.is_empty());
        assert_eq!(result[0]["tool"], "test");
    }

    #[test]
    fn test_extract_embedded_json() {
        let input = r#"I will run this tool: {"tool": "test", "args": {}}"#;
        let result = extract_tool_calls(input);
        assert!(!result.is_empty());
        assert_eq!(result[0]["tool"], "test");
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
        let result = extract_tool_calls(input);
        assert!(!result.is_empty());
        assert_eq!(result[0]["tool"], "test");
    }

    #[test]
    fn test_extract_suffix_text() {
        let input = r#"{"tool": "test", "args": {}} is the tool I used."#;
        let result = extract_tool_calls(input);
        assert!(!result.is_empty());
        assert_eq!(result[0]["tool"], "test");
    }

    #[test]
    fn test_extract_invalid_json() {
        let input = r#"This is just text with { brackets } but no valid json."#;
        let result = extract_tool_calls(input);
        assert!(result.is_empty());
    }

    #[test]
    fn test_extract_multiple_tools() {
        let input = r#"First tool: {"tool": "t1", "args": {}}, Second: {"tool": "t2", "args": {}}"#;
        let result = extract_tool_calls(input);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0]["tool"], "t1");
        assert_eq!(result[1]["tool"], "t2");
    }
}
