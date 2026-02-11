
//! Tests for mail system context retention
use anycowork::commands::mail::{get_mail_threads, send_mail, get_mail_thread_messages, reply_to_mail};
use anycowork::AppState;
use anyagents::database::create_test_pool;
use anyagents::models::{Agent, NewAgent};
use anyagents::schema::agents;
use diesel::prelude::*;
use std::sync::Arc;
use tauri::test::mock_builder;
use tauri::Manager;
use tokio::time::{sleep, Duration};

// Copied helpers...
fn create_test_agent(pool: &anyagents::database::DbPool, name: &str, avatar: &str, description: &str) -> String {
    let ai_config_json = serde_json::json!({
        "provider": "openai",
        "model": "gpt-4o"
    }).to_string();

    let agent_id = uuid::Uuid::new_v4().to_string();
    let new_agent = NewAgent {
        id: agent_id.clone(),
        name: name.to_string(),
        description: Some(description.to_string()),
        status: "active".to_string(),
        personality: Some("professional".to_string()),
        tone: Some("friendly".to_string()),
        expertise: Some("general".to_string()),
        ai_provider: "openai".to_string(),
        ai_model: "gpt-4o".to_string(),
        ai_temperature: 0.7,
        ai_config: ai_config_json,
        system_prompt: Some(format!("You are {}. {}", name, description)),
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
        avatar: Some(avatar.to_string()),
    };

    let mut conn = pool.get().expect("Failed to get DB connection");
    diesel::insert_into(agents::table)
        .values(&new_agent)
        .execute(&mut conn)
        .expect("Failed to insert test agent");

    agent_id
}

fn create_test_app_state() -> AppState {
    let pool = create_test_pool();
    AppState {
        db_pool: pool,
        pending_approvals: Arc::new(dashmap::DashMap::new()),
        telegram_manager: Arc::new(anycowork::telegram::TelegramBotManager::new(
            create_test_pool(),
        )),
        permission_manager: Arc::new(anyagents::permissions::PermissionManager::new()),
    }
}

fn create_app(state: AppState) -> tauri::App<tauri::test::MockRuntime> {
    let app = mock_builder()
        .build(tauri::generate_context!())
        .expect("error while running tauri application");
    app.manage(state);
    app
}

#[tokio::test]
async fn test_mail_context_retention() {
    dotenvy::dotenv().ok();
    let state = create_test_app_state();
    let app = create_app(state.clone());
    let state_handle = app.state::<AppState>();

    let agent_id = create_test_agent(
        &state.db_pool,
        "ContextBot",
        "🧠",
        "You are a helpful assistant with a good memory."
    );

    // 1. User sends a persistent fact
    let secret = "PurpleElephant";
    let thread = send_mail(
        state_handle.clone(),
        None,
        Some(agent_id.clone()),
        "Secret Code",
        format!("Remember this secret code: {}", secret),
    ).await.unwrap();

    // Wait for reply 1
    let mut reply1_received = false;
    for _ in 0..15 {
        sleep(Duration::from_secs(2)).await;
        let messages = get_mail_thread_messages(state_handle.clone(), thread.id.clone()).await.unwrap();
        if messages.len() > 1 {
            reply1_received = true;
            break;
        }
    }
    assert!(reply1_received, "Agent did not reply to first message");

    // 2. User asks for the fact
    let _ = reply_to_mail(
        state_handle.clone(),
        thread.id.clone(),
        None, // from user
        "What is the secret code I told you?".to_string(),
    ).await.unwrap();

    // Wait for reply 2
    let mut secret_found = false;
    for _ in 0..15 {
        sleep(Duration::from_secs(2)).await;
        let messages = get_mail_thread_messages(state_handle.clone(), thread.id.clone()).await.unwrap();
        // Should have: User, Agent, User, Agent (4 messages)
        if messages.len() >= 4 {
            let last_msg = messages.last().unwrap();
            println!("Last reply: {}", last_msg.content);
            if last_msg.content.contains(secret) {
                secret_found = true;
                break;
            }
        }
    }

    assert!(secret_found, "Agent failed to recall the secret code from context");
}
