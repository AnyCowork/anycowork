use crate::models::{Agent, AgentDto, NewAgent};
use crate::schema;
use crate::AppState;
use diesel::prelude::*;
use tauri::State;

#[tauri::command]
pub async fn create_agent(
    state: State<'_, AppState>,
    name: String,
    description: String,
    system_prompt: String,
) -> Result<AgentDto, String> {
    use schema::agents;

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    // Default AI config (could be passed in future)
    let ai_provider = "gemini".to_string();
    let ai_model = "gemini-3-flash-preview".to_string();
    let ai_config_json = serde_json::json!({
        "provider": ai_provider,
        "model": ai_model
    })
    .to_string();

    let new_agent = NewAgent {
        id: uuid::Uuid::new_v4().to_string(),
        name,
        description: Some(description),
        status: "active".to_string(),
        personality: None,
        tone: None,
        expertise: None,
        ai_provider,
        ai_model,
        ai_temperature: 0.7,
        ai_config: ai_config_json,
        system_prompt: Some(system_prompt),
        permissions: None,
        working_directories: None,
        skills: None,
        mcp_servers: None,
        messaging_connections: None,
        knowledge_bases: None,
        api_keys: None,
        created_at: chrono::Utc::now().timestamp(),
        updated_at: chrono::Utc::now().timestamp(),
        platform_configs: None,
        execution_settings: None,
        scope_type: None,
        workspace_path: None,
    };

    diesel::insert_into(agents::table)
        .values(&new_agent)
        .execute(&mut conn)
        .map_err(|e| e.to_string())?;

    let created_id = new_agent.id.to_string();

    let agent: Agent = agents::table
        .filter(agents::id.eq(created_id))
        .first::<Agent>(&mut conn)
        .map_err(|e| e.to_string())?;

    Ok(agent.into_dto())
}

#[tauri::command]
pub async fn get_agents(state: State<'_, AppState>) -> Result<Vec<AgentDto>, String> {
    use schema::agents::dsl::*;

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;
    let results = agents.load::<Agent>(&mut conn).map_err(|e| e.to_string())?;

    Ok(results.into_iter().map(|a| a.into_dto()).collect())
}

#[tauri::command]
pub async fn update_agent(
    state: State<'_, AppState>,
    agent_id: String,
    data: crate::models::AgentUpdateDto,
) -> Result<AgentDto, String> {
    use schema::agents::dsl::*;

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    // Fetch existing agent
    let mut agent = agents
        .filter(id.eq(&agent_id))
        .first::<Agent>(&mut conn)
        .map_err(|_| "Agent not found".to_string())?;

    // Check if this is the default agent - prevent skills/mcp changes
    let is_default_agent = agent.name == "AnyCoworker Default";

    // Update fields if present
    if let Some(n) = data.name {
        agent.name = n;
    }
    if let Some(d) = data.description {
        agent.description = Some(d);
    }
    // Status update logic if needed
    if let Some(s) = data.status {
        agent.status = s;
    }

    // Characteristics update
    if let Some(chars) = data.characteristics {
        agent.personality = chars.personality;
        agent.tone = chars.tone;
        agent.expertise = Some(chars.expertise.join(", "));
    }

    // AI Config update
    if let Some(config) = data.ai_config {
        agent.ai_provider = config.provider.clone(); // Update root fields too
        agent.ai_model = config.model.clone();
        agent.ai_temperature = config.temperature;

        let json_config = serde_json::to_string(&config).map_err(|e| e.to_string())?;
        agent.ai_config = json_config;
    }

    if let Some(prompt) = data.system_prompt {
        agent.system_prompt = Some(prompt);
    }

    // Skills and MCP servers - skip for default agent (it uses all enabled)
    if !is_default_agent {
        if let Some(s) = data.skills {
            agent.skills = Some(s.join(", "));

            // Sync agent_skill_assignments
            use crate::models::NewAgentSkillAssignment;
            use schema::agent_skill_assignments::dsl::{agent_skill_assignments, agent_id as col_agent_id};

            // 1. Delete existing assignments for this agent
            diesel::delete(agent_skill_assignments.filter(col_agent_id.eq(&agent_id)))
                .execute(&mut conn)
                .map_err(|e| format!("Failed to clear old skill assignments: {}", e))?;

            // 2. Insert new assignments
            if !s.is_empty() {
                 let new_assignments: Vec<NewAgentSkillAssignment> = s.iter().map(|sid| NewAgentSkillAssignment {
                    agent_id: agent_id.clone(),
                    skill_id: sid.clone(),
                    created_at: chrono::Utc::now().naive_utc(),
                }).collect();

                diesel::insert_into(agent_skill_assignments)
                    .values(&new_assignments)
                    .execute(&mut conn)
                    .map_err(|e| format!("Failed to insert new skill assignments: {}", e))?;
            }
        }

        if let Some(m) = data.mcp_servers {
            agent.mcp_servers = Some(m.join(", "));
        }
    }

    // Execution Settings
    if let Some(settings) = data.execution_settings {
        agent.execution_settings = Some(settings.to_string());
    }

    // Platform Configs - especially for Telegram
    if let Some(platform_configs_str) = data.platform_configs {
        agent.platform_configs = Some(platform_configs_str.clone());
    }

    agent.updated_at = chrono::Utc::now().timestamp();

    // Save changes
    diesel::update(agents.filter(id.eq(&agent_id)))
        .set((
            name.eq(&agent.name),
            description.eq(&agent.description),
            status.eq(&agent.status),
            personality.eq(&agent.personality),
            tone.eq(&agent.tone),
            expertise.eq(&agent.expertise),
            ai_provider.eq(&agent.ai_provider),
            ai_model.eq(&agent.ai_model),
            ai_temperature.eq(&agent.ai_temperature),
            ai_config.eq(&agent.ai_config),
            system_prompt.eq(&agent.system_prompt),
            skills.eq(&agent.skills),
            mcp_servers.eq(&agent.mcp_servers),
            execution_settings.eq(&agent.execution_settings),
            platform_configs.eq(&agent.platform_configs),
            updated_at.eq(&agent.updated_at),
        ))
        .execute(&mut conn)
        .map_err(|e| e.to_string())?;

    // Sync Telegram configuration if platform_configs changed
    if let Some(ref platform_configs_json) = agent.platform_configs {
        log::info!("Syncing Telegram config for agent {}", agent_id);
        match sync_agent_telegram_config(&state, &agent_id, platform_configs_json).await {
            Ok(_) => log::info!("Successfully synced Telegram config for agent {}", agent_id),
            Err(e) => log::error!("Failed to sync Telegram config for agent {}: {}", agent_id, e),
        }
    }

    Ok(agent.into_dto())
}

/// Synchronize Telegram configuration for an agent based on platform_configs
async fn sync_agent_telegram_config(
    state: &State<'_, AppState>,
    agent_id: &str,
    platform_configs_json: &str,
) -> Result<(), String> {
    use crate::models::{NewTelegramConfig, TelegramConfig};
    use schema::telegram_configs;

    // Parse platform_configs
    let platform_configs: serde_json::Value =
        serde_json::from_str(platform_configs_json).map_err(|e| format!("Failed to parse platform_configs: {}", e))?;

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    // Check for Telegram configuration
    if let Some(telegram) = platform_configs.get("telegram") {
        let bot_token = telegram.get("bot_token").and_then(|t| t.as_str());
        let enabled = telegram.get("enabled").and_then(|e| e.as_bool()).unwrap_or(false);

        if let Some(token) = bot_token {
            if !token.is_empty() {
                log::info!("Found Telegram bot token for agent {}, creating/updating config", agent_id);

                // Find existing telegram_config for this agent
                let existing: Option<TelegramConfig> = telegram_configs::table
                    .filter(telegram_configs::agent_id.eq(agent_id))
                    .first::<TelegramConfig>(&mut conn)
                    .optional()
                    .map_err(|e| e.to_string())?;

                let config_id = if let Some(existing_config) = existing {
                    log::info!("Updating existing Telegram config {} for agent {}", existing_config.id, agent_id);
                    
                    // Update if token changed
                    if existing_config.bot_token != token {
                        log::info!("Bot token changed for agent {}, stopping old bot and updating", agent_id);
                        // Stop old bot
                        let _ = state.telegram_manager.stop_bot(&existing_config.id).await;
                        
                        // Update token
                        diesel::update(telegram_configs::table.filter(telegram_configs::id.eq(&existing_config.id)))
                            .set((
                                telegram_configs::bot_token.eq(token),
                                telegram_configs::is_active.eq(if enabled { 1 } else { 0 }),
                                telegram_configs::updated_at.eq(chrono::Utc::now().naive_utc()),
                            ))
                            .execute(&mut conn)
                            .map_err(|e| e.to_string())?;
                    } else if existing_config.is_active != if enabled { 1 } else { 0 } {
                        // Only enabled status changed
                        diesel::update(telegram_configs::table.filter(telegram_configs::id.eq(&existing_config.id)))
                            .set((
                                telegram_configs::is_active.eq(if enabled { 1 } else { 0 }),
                                telegram_configs::updated_at.eq(chrono::Utc::now().naive_utc()),
                            ))
                            .execute(&mut conn)
                            .map_err(|e| e.to_string())?;
                    }

                    existing_config.id
                } else {
                    // Create new telegram_config
                    log::info!("Creating new Telegram config for agent {}", agent_id);
                    let new_config = NewTelegramConfig {
                        id: uuid::Uuid::new_v4().to_string(),
                        bot_token: token.to_string(),
                        agent_id: agent_id.to_string(),
                        is_active: if enabled { 1 } else { 0 },
                        allowed_chat_ids: None,
                        created_at: chrono::Utc::now().naive_utc(),
                        updated_at: chrono::Utc::now().naive_utc(),
                    };

                    diesel::insert_into(telegram_configs::table)
                        .values(&new_config)
                        .execute(&mut conn)
                        .map_err(|e| e.to_string())?;

                    new_config.id
                };

                // Start/stop bot based on enabled flag
                if enabled {
                    log::info!("Starting Telegram bot for agent {}", agent_id);
                    match state.telegram_manager.start_bot(&config_id).await {
                        Ok(_) => log::info!("✅ Telegram bot started for agent {}", agent_id),
                        Err(e) => log::error!("❌ Failed to start Telegram bot for agent {}: {}", agent_id, e),
                    }
                } else {
                    log::info!("Stopping Telegram bot for agent {} (disabled)", agent_id);
                    let _ = state.telegram_manager.stop_bot(&config_id).await;
                }

                return Ok(());
            }
        }
    }

    // No telegram config or empty token - delete existing config if any
    log::info!("No Telegram config for agent {}, cleaning up", agent_id);
    let existing_configs: Vec<TelegramConfig> = telegram_configs::table
        .filter(telegram_configs::agent_id.eq(agent_id))
        .load::<TelegramConfig>(&mut conn)
        .map_err(|e| e.to_string())?;

    for config in existing_configs {
        log::info!("Deleting Telegram config {} and stopping bot for agent {}", config.id, agent_id);
        let _ = state.telegram_manager.stop_bot(&config.id).await;
        diesel::delete(telegram_configs::table.filter(telegram_configs::id.eq(&config.id)))
            .execute(&mut conn)
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

use tauri::Runtime;

pub async fn chat_internal<R: Runtime>(
    window: tauri::WebviewWindow<R>,
    state: State<'_, AppState>,
    session_id: String,
    message: String,
    mode: Option<String>,
    model: Option<String>,
) -> Result<String, String> {
    use crate::models::Session;
    use schema::agents::dsl::agents;
    use schema::sessions::dsl::id as session_id_col;
    use schema::sessions::dsl::sessions;

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    // 1. Get Session to get Agent ID
    let session_record: Session = sessions
        .filter(session_id_col.eq(&session_id))
        .first(&mut conn)
        .map_err(|_| "Session not found".to_string())?;

    // 2. Get Agent
    let agent_record: Agent = agents
        .filter(schema::agents::dsl::id.eq(&session_record.agent_id))
        .first(&mut conn)
        .map_err(|_| "Agent not found".to_string())?;

    // 3. Save User Message
    use crate::models::NewMessage;
    use schema::messages;
    let user_msg = NewMessage {
        id: uuid::Uuid::new_v4().to_string(),
        role: "user".to_string(),
        content: message.clone(),
        session_id: session_id.clone(),
        metadata_json: None,
        tokens: None,
    };
    diesel::insert_into(messages::table)
        .values(&user_msg)
        .execute(&mut conn)
        .map_err(|e| e.to_string())?;

    // 4. Start Background Task
    crate::agents::start_chat_task(
        agent_record,
        message,
        session_id,
        window,
        state.pending_approvals.clone(),
        state.permission_manager.clone(),
        state.db_pool.clone(),
        mode.unwrap_or_else(|| "planning".to_string()),
        model,
    );

    Ok("started".to_string())
}

#[tauri::command]
pub async fn chat(
    window: tauri::WebviewWindow,
    state: State<'_, AppState>,
    session_id: String,
    message: String,
    mode: Option<String>,
    model: Option<String>,
) -> Result<String, String> {
    chat_internal(window, state, session_id, message, mode, model).await
}

#[tauri::command]
pub async fn approve_action(state: State<'_, AppState>, step_id: String) -> Result<(), String> {
    log::info!("approve_action called with step_id: {}", step_id);
    
    // Try PermissionManager first
    state.permission_manager.approve_request(&step_id);
    log::info!("Called permission_manager.approve_request");

    // Fallback to legacy pending_approvals if needed (optional)
    if let Some((_, tx)) = state.pending_approvals.remove(&step_id) {
        log::info!("Found in pending_approvals, sending true");
        let _ = tx.send(true);
    } else {
        log::info!("Not found in pending_approvals");
    }
    Ok(())
}

#[tauri::command]
pub async fn reject_action(state: State<'_, AppState>, step_id: String) -> Result<(), String> {
    log::info!("reject_action called with step_id: {}", step_id);
    
    // Try PermissionManager first
    state.permission_manager.reject_request(&step_id);

    // Fallback
    if let Some((_, tx)) = state.pending_approvals.remove(&step_id) {
        let _ = tx.send(false);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::create_test_pool;
    use crate::permissions::PermissionManager;
    use tauri::test::mock_builder;
    use tauri::Manager;

    fn create_app_state() -> crate::AppState {
        let pool = create_test_pool();
        crate::AppState {
            db_pool: pool,
            pending_approvals: std::sync::Arc::new(dashmap::DashMap::new()),
            telegram_manager: std::sync::Arc::new(crate::telegram::TelegramBotManager::new(
                create_test_pool(),
            )),
            permission_manager: std::sync::Arc::new(PermissionManager::new()),
        }
    }

    #[tokio::test]
    async fn test_create_and_get_agents() {
        let app = mock_builder().build(tauri::generate_context!()).unwrap();
        let state = create_app_state();
        app.manage(state);
        let state_handle = app.state::<crate::AppState>();

        // Test create_agent
        let agent = create_agent(
            state_handle.clone().into(),
            "Agent 1".to_string(),
            "Test Description".to_string(),
            "You are helpful".to_string(),
        )
        .await
        .unwrap();
        assert_eq!(agent.name, "Agent 1");

        // Test get_agents
        let agents = get_agents(state_handle.clone().into()).await.unwrap();
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].id, agent.id);
    }
}
