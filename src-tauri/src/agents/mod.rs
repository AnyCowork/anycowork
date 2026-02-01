// Refactored agents module using anycowork-core
// Only keep optimizations for now (might be useful for history management)
pub mod optimizations;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod workflow_tests;
#[cfg(test)]
mod skill_tests;

use crate::database::DbPool;
use crate::events::AgentEvent;
use crate::models::Agent as DbAgent;
use crate::permissions::PermissionManager;
use crate::tools::adapter::TauriBridgePermissionHandler;
use anycowork_core::agent::AgentCoordinator;
use anycowork_core::events::EventChannel;
use anycowork_core::permissions::PermissionManager as CorePermissionManager;
use anycowork_core::skills::SkillTool;
use anycowork_core::tools::{BashTool, FilesystemTool, SearchTool};
use diesel::prelude::*;
use log::error;
use rig::client::{CompletionClient, ProviderClient};
use rig::providers::{anthropic, gemini, openai};
use rig::tool::Tool;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use tauri::Emitter;
use tauri::Runtime;

pub struct AgentLoop {
    pub agent_db: DbAgent,
    pub session_id: String,
    pub db_pool: DbPool,
    pub workspace_path: std::path::PathBuf,
    pub permission_manager: Arc<PermissionManager>,
    pub events: Arc<EventChannel>,
    pub skills: Vec<SkillTool>,
}

impl AgentLoop {
    pub async fn new(agent_db: &DbAgent, db_pool: DbPool) -> Self {
        let workspace_path = if let Some(path) = &agent_db.workspace_path {
            std::path::PathBuf::from(path)
        } else {
            std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
        };

        std::fs::create_dir_all(&workspace_path).unwrap_or(());

        let permission_manager = Arc::new(PermissionManager::new());
        let events = Arc::new(EventChannel::new());

        // Load Skills
        let mut skills = Vec::new();
        if let Ok(mut conn) = db_pool.get() {
            use crate::schema::{agent_skill_assignments, agent_skills, skill_files};

            let skill_ids: Vec<String> = {
                let assignments = agent_skill_assignments::table
                    .filter(agent_skill_assignments::agent_id.eq(&agent_db.id))
                    .select(agent_skill_assignments::skill_id)
                    .load::<String>(&mut conn)
                    .unwrap_or_default();

                if !assignments.is_empty() {
                    assignments
                } else if agent_db.name == "AnyCoworker Default" {
                    agent_skills::table
                        .filter(agent_skills::enabled.eq(1))
                        .select(agent_skills::id)
                        .load::<String>(&mut conn)
                        .unwrap_or_default()
                } else {
                    Vec::new()
                }
            };

            if !skill_ids.is_empty() {
                log::info!("=== SKILL LOADING DEBUG START ===");
                log::info!("Agent: {}", agent_db.name);
                log::info!("Loading {} skill IDs", skill_ids.len());
                log::info!("Skill IDs: {:?}", skill_ids);
                
                let skills_list = agent_skills::table
                    .filter(agent_skills::id.eq_any(&skill_ids))
                    .filter(agent_skills::enabled.eq(1))
                    .load::<crate::models::AgentSkill>(&mut conn)
                    .unwrap_or_default();

                log::info!("Found {} skills from database", skills_list.len());
                log::info!("Skill names from DB: {:?}", skills_list.iter().map(|s| &s.name).collect::<Vec<_>>());

                let mut seen_names: HashSet<String> = HashSet::new();
                let sandbox = Arc::new(anycowork_core::sandbox::NativeSandbox::new());

                for skill_db in skills_list {
                    log::debug!("Processing skill: {}", skill_db.name);
                    
                    if !seen_names.insert(skill_db.name.clone()) {
                        log::warn!("!!! DUPLICATE DETECTED: Skipping duplicate skill: {}", skill_db.name);
                        continue;
                    }
                    
                    log::debug!("Loading skill: {}", skill_db.name);

                    let files_list = skill_files::table
                        .filter(skill_files::skill_id.eq(&skill_db.id))
                        .load::<crate::models::SkillFile>(&mut conn)
                        .unwrap_or_default();

                    let mut files_map = std::collections::HashMap::new();
                    for f in files_list {
                        files_map.insert(
                            f.relative_path,
                            anycowork_core::skills::SkillFileContent {
                                content: f.content,
                                file_type: f.file_type,
                            },
                        );
                    }

                    let sandbox_config = if let Some(sc_json) = &skill_db.sandbox_config {
                        serde_json::from_str(sc_json).ok()
                    } else {
                        None
                    };

                    let parsed_skill = anycowork_core::skills::ParsedSkill {
                        name: skill_db.name.clone(),
                        description: skill_db.description.clone(),
                        license: None,
                        triggers: None,
                        sandbox_config,
                        body: skill_db.skill_content.clone(),
                        category: skill_db.category.clone(),
                        requires_sandbox: skill_db.requires_sandbox == 1,
                        execution_mode: Some(skill_db.execution_mode.clone()),
                    };

                    let loaded_skill = anycowork_core::skills::LoadedSkill {
                        skill: parsed_skill,
                        files: files_map,
                    };

                    let skill_tool = SkillTool::new(
                        loaded_skill,
                        workspace_path.clone(),
                        anycowork_core::config::ExecutionMode::Flexible,
                        sandbox.clone(),
                    );
                    skills.push(skill_tool);
                }
                
                log::info!("Successfully loaded {} unique skills", skills.len());
            }
        }

        log::info!("AgentLoop initialized with {} skills for agent: {}", skills.len(), agent_db.name);

        Self {
            agent_db: agent_db.clone(),
            session_id: "temp".to_string(),
            db_pool,
            workspace_path,
            permission_manager,
            events,
            skills,
        }
    }

    pub async fn run<R: Runtime>(
        &mut self,
        user_message: String,
        job_id: String,
        window: tauri::WebviewWindow<R>,
        permission_manager: Arc<PermissionManager>,
        db_pool: DbPool,
    ) {
        self.session_id = job_id.clone();

        let events = self.events.clone();
        let session_id = self.session_id.clone();

        let window_clone = window.clone();
        let session_id_clone = session_id.clone();
        let db_pool_clone = db_pool.clone();
        let mut event_rx = events.subscribe();

        log::info!("Event channel receiver count: {}", events.receiver_count());

        // Spawn task to handle events and save thinking messages in real-time
        tokio::spawn(async move {
            use crate::models::NewMessage;
            use crate::schema::messages;
            
            log::info!("Event listener task started for session: {}", session_id_clone);
            
            while let Ok(event) = event_rx.recv().await {
                // Log all events for debugging
                log::info!("Event received: {:?}", event);
                
                // Save thinking messages to database immediately
                if let anycowork_core::events::AgentEvent::Thinking { ref message } = event {
                    if !message.is_empty() {
                        if let Ok(mut conn) = db_pool_clone.get() {
                            let thinking_msg = NewMessage {
                                id: uuid::Uuid::new_v4().to_string(),
                                role: "thinking".to_string(),
                                content: message.clone(),
                                session_id: session_id_clone.clone(),
                                metadata_json: Some(serde_json::json!({"type": "reasoning"}).to_string()),
                                tokens: None,
                            };
                            
                            if let Err(e) = diesel::insert_into(messages::table)
                                .values(&thinking_msg)
                                .execute(&mut conn)
                            {
                                log::error!("Failed to save thinking message: {}", e);
                            } else {
                                log::info!("Saved thinking message: {}", message);
                            }
                        }
                    }
                }

                let tauri_event = convert_core_event_to_tauri(event);
                log::info!("Emitting tauri event: {:?}", tauri_event);
                let _ = window_clone.emit(&format!("session:{}", session_id_clone), &tauri_event);
            }
            
            log::info!("Event listener task ended for session: {}", session_id_clone);
        });

        // Give the spawned task a moment to start
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let session_id_final = session_id.clone();
        let db_pool_final = db_pool.clone();

        let sandbox = Arc::new(anycowork_core::sandbox::NativeSandbox::new());
        let bridge = TauriBridgePermissionHandler::new(
            permission_manager.clone(),
            Some(window.clone()),
            self.session_id.clone(),
        );
        let core_permissions = Arc::new(CorePermissionManager::new(bridge));

        let fs_tool = FilesystemTool::new(self.workspace_path.clone(), core_permissions.clone());
        let bash_tool = BashTool::new(
            self.workspace_path.clone(),
            anycowork_core::config::ExecutionMode::Flexible,
            core_permissions.clone(),
            sandbox.clone(),
        );
        let search_tool = SearchTool::new(
            self.workspace_path.clone(),
            core_permissions.clone(),
            sandbox.clone(),
        );

        let provider = &self.agent_db.ai_provider;
        let model = &self.agent_db.ai_model;
        let preamble = self.agent_db.system_prompt.clone().unwrap_or_default();

        log::info!("=== AGENT BUILDING DEBUG ===");
        log::info!("Provider: {}, Model: {}", provider, model);
        log::info!("Skills to register: {}", self.skills.len());
        log::info!("Skill names: {:?}", self.skills.iter().map(|s| s.name()).collect::<Vec<_>>());

        let prompt_clone = user_message.clone();
        let job_state = Arc::new(Mutex::new(None));

        match provider.as_str() {
            "openai" => {
                let client = openai::Client::from_env();
                let mut builder = client
                    .agent(model)
                    .preamble(&preamble)
                    .tool(fs_tool)
                    .tool(bash_tool)
                    .tool(search_tool);

                for skill in &self.skills {
                    builder = builder.tool(skill.clone());
                }

                let agent = builder.build();
                let mut coordinator =
                    AgentCoordinator::new(agent, events.clone(), session_id.clone(), job_state);

                match coordinator.chat_stream(&prompt_clone).await {
                    Ok(response) => {
                        // Save assistant response to database
                        save_assistant_message(&db_pool_final, &session_id_final, &response).await;
                    }
                    Err(e) => {
                        error!("OpenAI chat stream error: {}", e);
                        let _ = window.emit(
                            &format!("session:{}", session_id),
                            AgentEvent::Error {
                                message: format!("Chat error: {}", e),
                                error: Some(e),
                            },
                        );
                    }
                }
            }
            "gemini" => {
                let client = gemini::Client::from_env();
                let mut builder = client
                    .agent(model)
                    .preamble(&preamble)
                    .tool(fs_tool)
                    .tool(bash_tool)
                    .tool(search_tool);

                log::info!("Adding {} skills to Gemini agent builder", self.skills.len());
                for (idx, skill) in self.skills.iter().enumerate() {
                    log::info!("  [{}] Adding skill: {}", idx + 1, skill.name());
                    builder = builder.tool(skill.clone());
                }
                log::info!("All skills added to builder");

                log::info!("Building Gemini agent...");
                let agent = builder.build();
                log::info!("Gemini agent built successfully");
                
                let mut coordinator =
                    AgentCoordinator::new(agent, events.clone(), session_id.clone(), job_state);

                match coordinator.chat_stream(&prompt_clone).await {
                    Ok(response) => {
                        // Save assistant response to database
                        save_assistant_message(&db_pool_final, &session_id_final, &response).await;
                    }
                    Err(e) => {
                        error!("Gemini chat stream error: {}", e);
                        let _ = window.emit(
                            &format!("session:{}", session_id),
                            AgentEvent::Error {
                                message: format!("Chat error: {}", e),
                                error: Some(e),
                            },
                        );
                    }
                }
            }
            "anthropic" => {
                let client = anthropic::Client::from_env();
                let mut builder = client
                    .agent(model)
                    .preamble(&preamble)
                    .tool(fs_tool)
                    .tool(bash_tool)
                    .tool(search_tool);

                for skill in &self.skills {
                    builder = builder.tool(skill.clone());
                }

                let agent = builder.build();
                let mut coordinator =
                    AgentCoordinator::new(agent, events.clone(), session_id.clone(), job_state);

                match coordinator.chat_stream(&prompt_clone).await {
                    Ok(response) => {
                        // Save assistant response to database
                        save_assistant_message(&db_pool_final, &session_id_final, &response).await;
                    }
                    Err(e) => {
                        error!("Anthropic chat stream error: {}", e);
                        let _ = window.emit(
                            &format!("session:{}", session_id),
                            AgentEvent::Error {
                                message: format!("Chat error: {}", e),
                                error: Some(e),
                            },
                        );
                    }
                }
            }
            _ => {
                error!("Unsupported provider: {}", provider);
                let _ = window.emit(
                    &format!("session:{}", session_id),
                    AgentEvent::Error {
                        message: format!("Unsupported provider: {}", provider),
                        error: None,
                    },
                );
            }
        }
    }
}

fn convert_core_event_to_tauri(event: anycowork_core::events::AgentEvent) -> AgentEvent {
    use anycowork_core::events::AgentEvent as CoreEvent;

    match event {
        CoreEvent::Token { content } => AgentEvent::Token { content },
        CoreEvent::JobStarted { job } => AgentEvent::JobStarted {
            job: convert_core_job_to_tauri(job),
        },
        CoreEvent::JobCompleted { job, message } => AgentEvent::JobCompleted {
            job: convert_core_job_to_tauri(job),
            message,
        },
        CoreEvent::StepStarted { job, step } => AgentEvent::StepStarted {
            job: convert_core_job_to_tauri(job),
            step: convert_core_step_to_tauri(step),
        },
        CoreEvent::StepCompleted { job, step } => AgentEvent::StepCompleted {
            job: convert_core_job_to_tauri(job),
            step: convert_core_step_to_tauri(step),
        },
        CoreEvent::ApprovalRequired { job, step } => AgentEvent::ApprovalRequired {
            job: convert_core_job_to_tauri(job),
            step: convert_core_step_to_tauri(step),
        },
        CoreEvent::StepApproved { job, step } => AgentEvent::StepApproved {
            job: convert_core_job_to_tauri(job),
            step: convert_core_step_to_tauri(step),
        },
        CoreEvent::StepRejected { job, step } => AgentEvent::StepRejected {
            job: convert_core_job_to_tauri(job),
            step: convert_core_step_to_tauri(step),
        },
        CoreEvent::Thinking { message } => AgentEvent::Thinking { message },
        CoreEvent::Error { message, error } => AgentEvent::Error { message, error },
        CoreEvent::PlanUpdate { plan } => AgentEvent::PlanUpdate {
            plan: crate::models::PlanUpdate {
                tasks: plan
                    .tasks
                    .into_iter()
                    .map(|t| crate::models::TaskState {
                        id: t.task_id,
                        description: t.description,
                        status: t.status,
                        result: t.result,
                    })
                    .collect(),
            },
        },
    }
}

fn convert_core_job_to_tauri(job: anycowork_core::events::ExecutionJob) -> crate::events::ExecutionJob {
    crate::events::ExecutionJob {
        id: job.id,
        session_id: job.session_id,
        status: job.status,
        query: job.query,
        steps: job
            .steps
            .into_iter()
            .map(convert_core_step_to_tauri)
            .collect(),
        current_step_index: job.current_step_index,
        created_at: job.created_at,
    }
}

fn convert_core_step_to_tauri(step: anycowork_core::events::ToolStep) -> crate::events::ExecutionStep {
    crate::events::ExecutionStep {
        id: step.id,
        tool_name: step.tool_name,
        tool_args: step.tool_args,
        status: step.status.to_string(),
        result: step.result,
        requires_approval: step.requires_approval,
        created_at: step.created_at,
    }
}

pub fn start_chat_task<R: Runtime>(
    agent: crate::models::Agent,
    message: String,
    session_id: String,
    window: tauri::WebviewWindow<R>,
    _pending_approvals: Arc<dashmap::DashMap<String, tokio::sync::oneshot::Sender<bool>>>,
    permission_manager: Arc<PermissionManager>,
    db_pool: DbPool,
    _mode: String,
    _model: Option<String>,
) {
    tokio::spawn(async move {
        let mut agent_loop = AgentLoop::new(&agent, db_pool.clone()).await;
        agent_loop
            .run(message, session_id, window, permission_manager, db_pool)
            .await;
    });
}

/// Save assistant message to database
async fn save_assistant_message(db_pool: &DbPool, session_id: &str, content: &str) {
    use crate::models::NewMessage;
    use crate::schema::messages;
    
    if content.is_empty() {
        return;
    }
    
    match db_pool.get() {
        Ok(mut conn) => {
            let assistant_msg = NewMessage {
                id: uuid::Uuid::new_v4().to_string(),
                role: "assistant".to_string(),
                content: content.to_string(),
                session_id: session_id.to_string(),
                metadata_json: None,
                tokens: None,
            };
            
            if let Err(e) = diesel::insert_into(messages::table)
                .values(&assistant_msg)
                .execute(&mut conn)
            {
                error!("Failed to save assistant message: {}", e);
            } else {
                log::info!("Saved assistant message");
            }
        }
        Err(e) => {
            error!("Failed to get database connection: {}", e);
        }
    }
}
