use tauri::test::{mock_builder, mock_context, noop_assets};
use tauri::{Manager, Emitter, App};
use std::sync::Arc;
use serde_json::json;
use tokio::time::{sleep, Duration};
use anycowork::AppState;
use anycowork::commands::agents::{create_agent, chat_internal, approve_action};
use anycowork::commands::sessions::create_session;
use anycowork::database::create_test_pool;
use anycowork::permissions::PermissionManager;


#[tokio::test]
async fn test_e2e_agent_interaction() {
    // 0. Load Env
    dotenvy::dotenv().ok();

    // 1. Setup App
    let app = mock_builder()
        .build(tauri::generate_context!())
        .expect("error while running tauri application");

    let pool = create_test_pool();
    let permission_manager = Arc::new(PermissionManager::new());
    let pending_approvals = Arc::new(dashmap::DashMap::new());
    
    let state = AppState {
        db_pool: pool.clone(),
        pending_approvals: pending_approvals.clone(),
        telegram_manager: Arc::new(anycowork::telegram::TelegramBotManager::new(pool.clone())),
        permission_manager: permission_manager.clone(),
    };

    app.manage(state);
    let state_handle = app.state::<AppState>();

    // 2. Create Agent
    let agent = create_agent(
        state_handle.clone().into(),
        "E2E Agent".to_string(),
        "Robot".to_string(),
        "You are a helpful assistant.".to_string(),
    ).await.expect("Failed to create agent");

    // 2.1 Create Session for that Agent
    let session = create_session(
        state_handle.clone().into(),
        agent.id.clone()
    ).await.expect("Failed to create session");

    // 3. Start Chat (this spawns the background task)
    let window = app.get_webview_window("main").unwrap_or_else(|| {
        tauri::WebviewWindowBuilder::new(&app, "main", tauri::WebviewUrl::App("index.html".into()))
            .build()
            .unwrap()
    });

    let session_id = session.id.clone();
    let test_file_path = "e2e_test_file.txt";
    let file_path_check = test_file_path.to_string();
    
    // Clean up previous run
    let _ = std::fs::remove_file(test_file_path);

    // Send message to create file
    let result = chat_internal(
        window.clone(), 
        state_handle.clone().into(),
        session_id.clone(),
        format!("Create a file named {} with content 'interaction_verified'", test_file_path)
    ).await;
    
    if let Err(e) = &result {
        println!("chat_internal error: {}", e);
    }
    assert!(result.is_ok());

    // 4. Poll for permission request and approve it
    let pm = permission_manager.clone();
    tokio::spawn(async move {
        // Poll for up to 60 seconds (matching file wait)
        for i in 0..60 {
            let requests = pm.get_pending_requests();
            if !requests.is_empty() {
                println!("E2E Poll: Found pending requests: {:?}", requests);
                for req_id in requests {
                    println!("E2E Poll: Approving request: {}", req_id); 
                    pm.approve_request(&req_id);
                }
            } else if i % 5 == 0 {
                println!("E2E Poll: No requests found (iteration {})", i);
            }
            sleep(Duration::from_secs(1)).await;
        }
        println!("E2E Poll: Finished polling.");
    });

    // We need to wait for the file to exist.
    let mut file_exists = false;
    for _ in 0..60 { // Wait up to 60s (LLM might be slow)
        if std::fs::metadata(&file_path_check).is_ok() {
            file_exists = true;
            break;
        }
        sleep(Duration::from_secs(1)).await;
    }
    
    assert!(file_exists, "File was not created by the agent");

    // Cleanup
    let _ = std::fs::remove_file(file_path_check);
}
