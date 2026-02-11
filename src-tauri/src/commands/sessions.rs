use anyagents::models::{Message, NewMessage, NewSession, Session, UpdateSession};
use anyagents::schema;
use crate::AppState;
use diesel::prelude::*;
use serde::Serialize;
use tauri::State;

#[tauri::command]
pub async fn create_session(
    state: State<'_, AppState>,
    agent_id: String,
) -> Result<Session, String> {
    use anyagents::schema::{agents, sessions};

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    // Get agent name to create default title
    let agent = agents::table
        .filter(agents::id.eq(&agent_id))
        .first::<anyagents::models::Agent>(&mut conn)
        .map_err(|e| format!("Agent not found: {}", e))?;

    // Create default title: "Chat with {agent_name}"
    let default_title = format!("Chat with {}", agent.name);

    let new_session = NewSession {
        id: uuid::Uuid::new_v4().to_string(),
        agent_id,
        title: Some(default_title),
        created_at: chrono::Utc::now().naive_utc(),
        updated_at: chrono::Utc::now().naive_utc(),
        archived: 0,
        pinned: 0,
    };

    diesel::insert_into(sessions::table)
        .values(&new_session)
        .execute(&mut conn)
        .map_err(|e| e.to_string())?;

    let created_id = new_session.id.clone();

    sessions::table
        .filter(sessions::id.eq(created_id))
        .first::<Session>(&mut conn)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_sessions(
    state: State<'_, AppState>,
    archived_param: Option<bool>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<Session>, String> {
    use anyagents::schema::sessions::dsl::{archived, pinned, sessions, updated_at};

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    let mut query = sessions.into_boxed();

    // Filter by archived status if specified
    if let Some(arch) = archived_param {
        let arch_val = if arch { 1 } else { 0 };
        query = query.filter(archived.eq(arch_val));
    }

    // Order by pinned first, then by updated_at
    query = query.order((pinned.desc(), updated_at.desc()));

    // Apply pagination if specified
    if let Some(lim) = limit {
        query = query.limit(lim);
    }
    if let Some(off) = offset {
        query = query.offset(off);
    }

    query.load::<Session>(&mut conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_session(state: State<'_, AppState>, session_id: String) -> Result<(), String> {
    use anyagents::schema::sessions::dsl::*;

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;
    diesel::delete(sessions.filter(id.eq(session_id)))
        .execute(&mut conn)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn get_session_messages(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<Vec<Message>, String> {
    use anyagents::schema::messages::dsl::{created_at, messages};

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;
    let results = messages
        .filter(schema::messages::dsl::session_id.eq(session_id))
        .order(created_at.asc())
        .load::<Message>(&mut conn)
        .map_err(|e| e.to_string())?;

    Ok(results)
}

// ============================================================================
// NEW PHASE 2 COMMANDS
// ============================================================================

#[tauri::command]
pub async fn update_session(
    state: State<'_, AppState>,
    session_id: String,
    title: Option<String>,
    archived_param: Option<bool>,
    pinned_param: Option<bool>,
) -> Result<Session, String> {
    use anyagents::schema::sessions::dsl::{id, sessions};

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    let update = UpdateSession {
        title,
        archived: archived_param.map(|v| if v { 1 } else { 0 }),
        pinned: pinned_param.map(|v| if v { 1 } else { 0 }),
        updated_at: chrono::Utc::now().naive_utc(),
    };

    diesel::update(sessions.filter(id.eq(&session_id)))
        .set(&update)
        .execute(&mut conn)
        .map_err(|e| e.to_string())?;

    sessions
        .filter(id.eq(&session_id))
        .first::<Session>(&mut conn)
        .map_err(|e| e.to_string())
}

#[derive(Serialize)]
pub struct SessionWithMessages {
    pub session: Session,
    pub messages: Vec<Message>,
}

#[tauri::command]
pub async fn get_session_with_messages(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<SessionWithMessages, String> {
    use anyagents::schema::messages::dsl::{created_at, messages, session_id as msg_session_id};
    use anyagents::schema::sessions::dsl::{id as session_id_col, sessions};

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    // Get session
    let session = sessions
        .filter(session_id_col.eq(&session_id))
        .first::<Session>(&mut conn)
        .map_err(|e| format!("Session not found: {}", e))?;

    // Get messages
    let msgs = messages
        .filter(msg_session_id.eq(&session_id))
        .order(created_at.asc())
        .load::<Message>(&mut conn)
        .map_err(|e| e.to_string())?;

    Ok(SessionWithMessages {
        session,
        messages: msgs,
    })
}

#[tauri::command]
pub async fn add_message(
    state: State<'_, AppState>,
    session_id: String,
    role: String,
    content: String,
    metadata_json: Option<String>,
    tokens: Option<i32>,
) -> Result<Message, String> {
    use anyagents::schema::messages;

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    let new_message = NewMessage {
        id: uuid::Uuid::new_v4().to_string(),
        role,
        content,
        session_id: session_id.clone(),
        metadata_json,
        tokens,
    };

    diesel::insert_into(messages::table)
        .values(&new_message)
        .execute(&mut conn)
        .map_err(|e| e.to_string())?;

    let created_id = new_message.id.clone();

    messages::table
        .filter(messages::id.eq(created_id))
        .first::<Message>(&mut conn)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_message(state: State<'_, AppState>, message_id: String) -> Result<(), String> {
    use anyagents::schema::messages::dsl::{id, messages};

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;
    diesel::delete(messages.filter(id.eq(message_id)))
        .execute(&mut conn)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[derive(Serialize)]
pub struct SessionStats {
    pub session_id: String,
    pub message_count: i64,
    pub total_tokens: Option<i64>,
}

#[tauri::command]
pub async fn get_session_stats(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<SessionStats, String> {
    use anyagents::schema::messages::dsl::{messages, session_id as msg_session_id, tokens};

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    // Count messages
    let count: i64 = messages
        .filter(msg_session_id.eq(&session_id))
        .count()
        .get_result(&mut conn)
        .map_err(|e| e.to_string())?;

    // Sum tokens (if available)
    let total_tokens: Option<i64> = messages
        .filter(msg_session_id.eq(&session_id))
        .select(diesel::dsl::sum(tokens))
        .first(&mut conn)
        .ok()
        .flatten();

    Ok(SessionStats {
        session_id,
        message_count: count,
        total_tokens,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyagents::database::create_test_pool;
    use anyagents::models::NewAgent;
    use anyagents::schema::agents;
    use tauri::test::mock_builder;
    use tauri::Manager;

    // Helper to create a test agent needed for session creation
    fn create_test_agent(conn: &mut SqliteConnection) -> String {
        let agent_id = uuid::Uuid::new_v4().to_string();
        let new_agent = NewAgent {
            id: agent_id.clone(),
            name: "Test Agent".to_string(),
            description: None,
            status: "active".to_string(),
            personality: None,
            tone: None,
            expertise: None,
            ai_provider: "openai".to_string(),
            ai_model: "gpt-4".to_string(),
            ai_temperature: 0.7,
            ai_config: "{}".to_string(),
            system_prompt: None,
            permissions: None,
            working_directories: None,
            skills: None,
            mcp_servers: None,
            messaging_connections: None,
            knowledge_bases: None,
            api_keys: None,
            created_at: 0,
            updated_at: 0,
            platform_configs: None,
            execution_settings: None,
            scope_type: None,
            workspace_path: None,
            avatar: None,
        };

        diesel::insert_into(agents::table)
            .values(&new_agent)
            .execute(conn)
            .unwrap();

        agent_id
    }

    fn create_app_state() -> crate::AppState {
        let pool = create_test_pool();
        crate::AppState {
            db_pool: pool,
            pending_approvals: std::sync::Arc::new(dashmap::DashMap::new()),
            telegram_manager: std::sync::Arc::new(crate::telegram::TelegramBotManager::new(
                create_test_pool(),
            )),
            permission_manager: std::sync::Arc::new(anyagents::permissions::PermissionManager::new()),
        }
    }

    #[tokio::test]
    async fn test_create_and_get_sessions() {
        let app = mock_builder().build(tauri::generate_context!()).unwrap();
        let state = create_app_state();
        app.manage(state);
        let state_handle = app.state::<crate::AppState>();

        let mut conn = state_handle.db_pool.get().unwrap();
        let agent_id = create_test_agent(&mut conn);

        // Test create_session
        let session = create_session(state_handle.clone().into(), agent_id.clone())
            .await
            .unwrap();
        assert_eq!(session.agent_id, agent_id);

        // Test get_sessions
        let sessions = get_sessions(state_handle.clone().into(), Some(false), None, None)
            .await
            .unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].id, session.id);
    }

    #[tokio::test]
    async fn test_add_and_get_messages() {
        let app = mock_builder().build(tauri::generate_context!()).unwrap();
        let state = create_app_state();
        app.manage(state);
        let state_handle = app.state::<crate::AppState>();

        let mut conn = state_handle.db_pool.get().unwrap();
        let agent_id = create_test_agent(&mut conn);
        let session = create_session(state_handle.clone().into(), agent_id)
            .await
            .unwrap();

        // Test add_message
        let msg = add_message(
            state_handle.clone().into(),
            session.id.clone(),
            "user".to_string(),
            "Hello".to_string(),
            None,
            None,
        )
        .await
        .unwrap();
        assert_eq!(msg.content, "Hello");

        // Test get_session_messages
        let messages = get_session_messages(state_handle.clone().into(), session.id)
            .await
            .unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].id, msg.id);
    }
}
