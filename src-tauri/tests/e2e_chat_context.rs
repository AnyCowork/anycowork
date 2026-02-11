
//! Tests for chat system context retention (AgentLoop/Planner)
use anycowork::commands::agents::{chat_internal, create_agent};
use anycowork::commands::sessions::create_session;
use anyagents::database::create_test_pool;
use anyagents::permissions::PermissionManager;
use anycowork::AppState;
use std::sync::Arc;
use tauri::test::mock_builder;
use tauri::Manager;
use tokio::time::{sleep, Duration};

fn create_app(state: AppState) -> tauri::App<tauri::test::MockRuntime> {
    let app = mock_builder()
        .build(tauri::generate_context!())
        .expect("error while running tauri application");
    app.manage(state);
    app
}

#[tokio::test]
async fn test_chat_context_retention() {
    dotenvy::dotenv().ok();
    
    // Setup
    let pool = create_test_pool();
    let permission_manager = Arc::new(PermissionManager::new());
    let pending_approvals = Arc::new(dashmap::DashMap::new());
    
    let state = AppState {
        db_pool: pool.clone(),
        pending_approvals: pending_approvals.clone(),
        telegram_manager: Arc::new(anycowork::telegram::TelegramBotManager::new(pool.clone())),
        permission_manager: permission_manager.clone(),
    };

    let app = create_app(state.clone());
    let state_handle = app.state::<AppState>();

    // Create Agent with fast mode to use AgentLoop directly (or flexible which uses Router)
    // We want to test AgentLoop context, so let's try to trigger it.
    // "My secret is X" is Simple Chat (handled by SimpleChatAgent).
    // "Create a file with my secret" is Complex/Tool (handled by AgentLoop).
    // IF SimpleChatAgent saves to DB, and AgentLoop reads from DB, it works.
    // BUT AgentLoop currently does NOT read from DB.

    let agent = create_agent(
        state_handle.clone().into(),
        "ContextBot".to_string(),
        "MemoryTester".to_string(),
        "You are a helpful assistant.".to_string(),
    ).await.expect("Failed to create agent");

    let session = create_session(state_handle.clone().into(), agent.id.clone())
        .await
        .expect("Failed to create session");

    // Mock Window
    let window = tauri::WebviewWindowBuilder::new(&app, "main", tauri::WebviewUrl::App("index.html".into()))
        .build()
        .unwrap();

    let session_id = session.id.clone();
    let unique_secret = format!("Secret-{}", uuid::Uuid::new_v4());
    let filename = format!("secret_{}.txt", uuid::Uuid::new_v4());
    
    // 1. Tell secret (Simple Chat likely)
    let _ = chat_internal(
        window.clone(),
        state_handle.clone().into(),
        session_id.clone(),
        format!("My secret code is {}", unique_secret),
        Some("auto".to_string()),
        None,
    ).await.expect("Chat 1 failed");

    // Wait for processing
    sleep(Duration::from_secs(5)).await;

    // 2. Ask to use secret in tool (AgentLoop)
    // This forces AgentLoop execution. If it doesn't load history, it won't know the secret.
    let _ = chat_internal(
        window.clone(),
        state_handle.clone().into(),
        session_id.clone(),
        format!("Create a file named {} containing ONLY my secret code.", filename),
        Some("auto".to_string()),
        None,
    ).await.expect("Chat 2 failed");

    // Poll for file creation (max 30s)
    let mut file_created = false;
    for _ in 0..15 {
        if std::path::Path::new(&filename).exists() {
            file_created = true;
            break;
        }
        sleep(Duration::from_secs(2)).await;
    }

    assert!(file_created, "Agent failed to create file (likely context loss or tool failure)");

    let content = std::fs::read_to_string(&filename).unwrap_or_default();
    std::fs::remove_file(&filename).unwrap_or(()); // Cleanup

    assert!(content.contains(&unique_secret), "File content '{}' did not contain secret '{}'", content, unique_secret);
}
