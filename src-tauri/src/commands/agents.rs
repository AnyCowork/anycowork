use tauri::State;
use crate::AppState;
use crate::models::{NewAgent, Agent, AgentDto};
use crate::schema;
use diesel::prelude::*;

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
    let ai_provider = "openai".to_string();
    let ai_model = "gpt-4o".to_string();
    let ai_config_json = serde_json::json!({
        "provider": ai_provider,
        "model": ai_model
    }).to_string();

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
    let results = agents
        .load::<Agent>(&mut conn)
        .map_err(|e| e.to_string())?;

    Ok(results.into_iter().map(|a| a.into_dto()).collect())
}

use tauri::Runtime;

pub async fn chat_internal<R: Runtime>(
    window: tauri::WebviewWindow<R>,
    state: State<'_, AppState>,
    session_id: String,
    message: String,
) -> Result<String, String> {
    use schema::sessions::dsl::id as session_id_col;
    use schema::sessions::dsl::sessions;
    use schema::agents::dsl::agents;
    use crate::models::Session;

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
    use schema::messages;
    use crate::models::NewMessage;
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

    // 4. Create AgentLoop
    let loop_instance = crate::agents::AgentLoop::<R>::new(&agent_record).await;

    // 5. Start Background Task
    crate::agents::start_chat_task(
        loop_instance,
        message,
        session_id,
        window,
        state.pending_approvals.clone(),
        state.permission_manager.clone(), // Add permission manager
        state.db_pool.clone(),
    );

    Ok("started".to_string())
}

#[tauri::command]
pub async fn chat(
    window: tauri::WebviewWindow,
    state: State<'_, AppState>,
    session_id: String,
    message: String,
) -> Result<String, String> {
    chat_internal(window, state, session_id, message).await
}

#[tauri::command]
pub async fn approve_action(state: State<'_, AppState>, step_id: String) -> Result<(), String> {
    // Try PermissionManager first
    state.permission_manager.approve_request(&step_id);

    // Fallback to legacy pending_approvals if needed (optional)
    if let Some((_, tx)) = state.pending_approvals.remove(&step_id) {
        let _ = tx.send(true);
    }
    Ok(())
}

#[tauri::command]
pub async fn reject_action(state: State<'_, AppState>, step_id: String) -> Result<(), String> {
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
    use tauri::test::mock_builder;
    use tauri::Manager;
    use crate::permissions::PermissionManager;

    fn create_app_state() -> crate::AppState {
        let pool = create_test_pool();
        crate::AppState {
            db_pool: pool,
            pending_approvals: std::sync::Arc::new(dashmap::DashMap::new()),
            telegram_manager: std::sync::Arc::new(crate::telegram::TelegramBotManager::new(create_test_pool())),
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
        ).await.unwrap();
        assert_eq!(agent.name, "Agent 1");

        // Test get_agents
        let agents = get_agents(state_handle.clone().into()).await.unwrap();
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].id, agent.id);
    }
}
