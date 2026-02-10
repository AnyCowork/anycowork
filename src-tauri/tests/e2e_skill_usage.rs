use anycowork::commands::agents::{chat_internal, create_agent};
use anycowork::commands::sessions::create_session;
use anyagents::database::create_test_pool;
use anyagents::models::{NewAgentSkill, NewAgentSkillAssignment};
use anyagents::permissions::PermissionManager;
use anyagents::schema::{agent_skill_assignments, agent_skills};
use anycowork::AppState;
use diesel::prelude::*;
use std::sync::Arc;
use tauri::test::mock_builder;
use tauri::Manager;
use tokio::time::{sleep, Duration};
use uuid::Uuid;

// Shared setup helper
// Shared setup helper
async fn setup_app_and_agent(skill_name: &str, skill_content: &str, requires_sandbox: bool, execution_mode: &str, workspace_path: String) -> (tauri::App<tauri::test::MockRuntime>, AppState, String, String) {
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

    app.manage(state.clone());
    let state_handle = app.state::<AppState>();

    // Create Agent
    let mut agent = create_agent(
        state_handle.clone().into(),
        "Skill Agent".to_string(),
        "Tester".to_string(),
        format!("You are a helpful assistant testing the {} skill. Always use tools. The information you need is in the {} skill.", skill_name, skill_name),
    )
    .await
    .expect("Failed to create agent");
    
    // Update workspace_path in DB manually
    let mut conn = pool.get().unwrap();
    use anyagents::schema::agents;
    use diesel::prelude::*;
    diesel::update(agents::table.filter(agents::id.eq(&agent.id)))
        .set(agents::workspace_path.eq(&workspace_path))
        .execute(&mut conn)
        .unwrap();

    // Insert Skill
    let skill_id = Uuid::new_v4().to_string();
    let new_skill = NewAgentSkill {
        id: skill_id.clone(),
        name: skill_name.to_string(),
        display_title: skill_name.to_string(),
        description: format!("Use this skill to access information about {}.", skill_name),
        skill_content: skill_content.to_string(),
        additional_files_json: None,
        enabled: 1,
        version: 1,
        created_at: chrono::Utc::now().naive_utc(),
        updated_at: chrono::Utc::now().naive_utc(),
        source_path: None,
        category: Some("Test".to_string()),
        requires_sandbox: if requires_sandbox { 1 } else { 0 },
        sandbox_config: None,
        execution_mode: execution_mode.to_string(),
    };

    diesel::insert_into(agent_skills::table)
        .values(&new_skill)
        .execute(&mut conn)
        .unwrap();

    // Assign Skill
    let assignment = NewAgentSkillAssignment {
        agent_id: agent.id.clone(),
        skill_id: skill_id.clone(),
        created_at: chrono::Utc::now().naive_utc(),
    };

    diesel::insert_into(agent_skill_assignments::table)
        .values(&assignment)
        .execute(&mut conn)
        .unwrap();

    let session = create_session(state_handle.clone().into(), agent.id.clone())
        .await
        .expect("Failed to create session");

    (app, state, session.id, agent.id)
}

fn start_permission_poller(pm: Arc<PermissionManager>) {
    tokio::spawn(async move {
        for _ in 0..60 { // Poll for 60s
            let requests = pm.get_pending_requests();
            for req_id in requests {
                println!("Auto-approving request: {}", req_id);
                pm.approve_request(&req_id);
            }
            sleep(Duration::from_millis(500)).await;
        }
    });
}

#[tokio::test]
async fn test_e2e_skill_action_execution() {
    dotenvy::dotenv().ok();
    if std::env::var("GEMINI_API_KEY").is_err() {
        println!("Skipping E2E test due to missing GEMINI_API_KEY");
        return;
    }

    let file_name = "skill_e2e_test.txt";
    
    // Create temp dir for workspace
    let temp_dir = tempfile::Builder::new().prefix("agent_e2e_ws_action_").tempdir().expect("Failed to create temp dir");
    let workspace_path = temp_dir.path().to_string_lossy().to_string();

    let (app, state, session_id, _) = setup_app_and_agent(
        "file-maker", 
        "To make a file, use the command 'touch <filename>'.", 
        false,
        "direct",
        workspace_path.clone()
    ).await;

    start_permission_poller(state.permission_manager.clone());

    let window = app.get_webview_window("main").unwrap_or_else(|| {
        tauri::WebviewWindowBuilder::new(&app, "main", tauri::WebviewUrl::App("index.html".into()))
            .build()
            .unwrap()
    });

    // Send chat
    let result = chat_internal(
        window,
        app.state::<AppState>().clone().into(),
        session_id.clone(),
        format!("Use the file-maker skill to create a file named {}", file_name),
        None,
        None
    ).await;

    assert!(result.is_ok());

    // Wait for file in temp dir
    let target_path = temp_dir.path().join(file_name);
    let mut exists = false;
    for _ in 0..60 {
        if target_path.exists() {
            exists = true;
            break;
        }
        sleep(Duration::from_secs(1)).await;
    }

    assert!(exists, "File was not created by the skill in workspace: {}", workspace_path);
}

#[tokio::test]
async fn test_e2e_skill_knowledge_read() {
    dotenvy::dotenv().ok();
    if std::env::var("GEMINI_API_KEY").is_err() {
        return;
    }
    
    // Create temp dir for workspace
    let temp_dir = tempfile::Builder::new().prefix("agent_e2e_ws_read_").tempdir().expect("Failed to create temp dir");
    let workspace_path = temp_dir.path().to_string_lossy().to_string();

    let (app, state, session_id, _) = setup_app_and_agent(
        "secret-keeper", 
        "The secret code is 'BLUE_MOON'.", 
        false,
        "direct",
        workspace_path
    ).await;
    
    start_permission_poller(state.permission_manager.clone());
    
    let window = app.get_webview_window("main").unwrap_or_else(|| {
        tauri::WebviewWindowBuilder::new(&app, "main", tauri::WebviewUrl::App("index.html".into()))
            .build()
            .unwrap()
    });

    // Ask for the secret
    let _ = chat_internal(
        window,
        app.state::<AppState>().clone().into(),
        session_id.clone(),
        "What is the secret code mentioned in the secret-keeper skill?".to_string(),
        None,
        None
    ).await;

    // Check DB
    let pool = state.db_pool.clone();
    let mut found = false;
    use anyagents::schema::messages::dsl::{messages, session_id as session_id_col, role, created_at};
    
    for _ in 0..60 {
        let mut conn = pool.get().unwrap();
        let msgs = messages
            .filter(session_id_col.eq(&session_id))
            .order(created_at.asc())
            .load::<anyagents::models::Message>(&mut conn)
            .unwrap();
            
        for msg in msgs {
            println!("[{}] {}", msg.role, msg.content);
            if msg.role == "assistant" && msg.content.contains("secret-keeper") && msg.content.contains("read") {
                 found = true;
            }
            if msg.content.contains("BLUE_MOON") {
                found = true;
            }
        }
        if found { break; }
        sleep(Duration::from_secs(1)).await;
    }
    
    assert!(found, "Agent did not use the skill (read) or find the secret");
}

#[tokio::test]
async fn test_e2e_skill_error_handling() {
    dotenvy::dotenv().ok();
    if std::env::var("GEMINI_API_KEY").is_err() {
        return;
    }
    
    let temp_dir = tempfile::Builder::new().prefix("agent_e2e_ws_error_").tempdir().expect("Failed to create temp dir");
    let workspace_path = temp_dir.path().to_string_lossy().to_string();

    let (app, state, session_id, _) = setup_app_and_agent(
        "broken-tool", 
        "This tool always fails. Use command 'fail' to trigger error.", 
        false,
        "direct",
        workspace_path
    ).await;
    
    start_permission_poller(state.permission_manager.clone());
    
    let window = app.get_webview_window("main").unwrap_or_else(|| {
        tauri::WebviewWindowBuilder::new(&app, "main", tauri::WebviewUrl::App("index.html".into()))
            .build()
            .unwrap()
    });
    
    // Send command that will fail (we assume the tool fails naturally or we check how agent handles missing tool/invalid args)
    // Actually, we need to make sure the TOOL itself returns an error.
    // Our setup currently creates a skill with static content. `SkillTool` executes by looking for `args`. 
    // If we pass an invalid command (not 'read'), it typically tries to run as bash script or docker.
    // If execution_mode is direct, it tries running as script.
    // So "fail" will try to run `fail` command in bash. `fail` likely doesn't exist.
    
    let _ = chat_internal(
        window,
        app.state::<AppState>().clone().into(),
        session_id.clone(),
        "Run the broken-tool with command 'fail'. Report the error back to me.".to_string(),
        None,
        None
    ).await;

    // Check DB for assistant mentioning error
    let pool = state.db_pool.clone();
    let mut found = false;
    use anyagents::schema::messages::dsl::{messages, session_id as session_id_col, role, created_at};
    
    for _ in 0..60 {
        let mut conn = pool.get().unwrap();
        let msgs = messages
            .filter(session_id_col.eq(&session_id))
            .order(created_at.asc())
            .load::<anyagents::models::Message>(&mut conn)
            .unwrap();
            
        for msg in msgs {
            if msg.role == "tool" && msg.content.contains("Error") {
                 found = true;
            }
        }
        if found { break; }
        sleep(Duration::from_secs(1)).await;
    }
    
    assert!(found, "System did not record the tool error");
}

#[tokio::test]
async fn test_e2e_skill_sandbox_execution() {
    dotenvy::dotenv().ok();
    if std::env::var("GEMINI_API_KEY").is_err() {
        return;
    }
    
    // Check docker availability first, skip if not present
    let output = std::process::Command::new("docker").arg("--version").output();
    if output.is_err() || !output.unwrap().status.success() {
        println!("Skipping sandbox test due to missing Docker");
        return;
    }

    let file_name = "sandbox_file.txt";
    
    // Use a temp dir that fits Docker volume mounting requirements if needed.
    // Standard temp dirs usually work on Linux.
    let temp_dir = tempfile::Builder::new().prefix("agent_e2e_ws_sandbox_").tempdir().expect("Failed to create temp dir");
    let workspace_path = temp_dir.path().to_string_lossy().to_string();
    
    // Ensure dir exists and is accessible
    let _ = std::fs::create_dir_all(&workspace_path);

    let (app, state, session_id, _) = setup_app_and_agent(
        "sandbox-maker", 
        "To make a file in sandbox, use the command: echo 'sandbox' > sandbox_file.txt",
        true, // requires_sandbox
        "sandbox",
        workspace_path.clone()
    ).await;
    
    start_permission_poller(state.permission_manager.clone());
    
    let window = app.get_webview_window("main").unwrap_or_else(|| {
        tauri::WebviewWindowBuilder::new(&app, "main", tauri::WebviewUrl::App("index.html".into()))
            .build()
            .unwrap()
    });

    // Send chat
    let _ = chat_internal(
        window,
        app.state::<AppState>().clone().into(),
        session_id.clone(),
        "Use the sandbox-maker skill to create 'sandbox_file.txt' with content 'sandbox'. confirm when done.".to_string(),
        None,
        None
    ).await;

    // With sandbox, the file should be created in the WORKSPACE path (mapped volume)
    let target_path = temp_dir.path().join(file_name);
    let mut exists = false;
    for _ in 0..120 { // Docker might take longer to pull image/start
        if target_path.exists() {
            exists = true;
            break;
        }
        sleep(Duration::from_secs(1)).await;
    }

    assert!(exists, "File was not created by the skill in sandbox workspace: {}", workspace_path);
}

#[tokio::test]
async fn test_e2e_skill_sandbox_isolation() {
    dotenvy::dotenv().ok();
    if std::env::var("GEMINI_API_KEY").is_err() {
        return;
    }
    
    // Check docker availability
    let output = std::process::Command::new("docker").arg("--version").output();
    if output.is_err() || !output.unwrap().status.success() {
         return;
    }

    // 1. Create a secret file OUTSIDE the workspace
    let secret_dir = tempfile::Builder::new().prefix("secret_outside_").tempdir().expect("Failed to create secret dir");
    let secret_path = secret_dir.path().join("secret.txt");
    std::fs::write(&secret_path, "TOP_SECRET_HOST_DATA").expect("Failed to write secret");
    let secret_path_str = secret_path.to_string_lossy().to_string();
    
    // 2. Create Workspace
    let workspace_dir = tempfile::Builder::new().prefix("agent_e2e_ws_isolation_").tempdir().expect("Failed to create ws dir");
    let workspace_path = workspace_dir.path().to_string_lossy().to_string();

    let (app, state, session_id, _) = setup_app_and_agent(
        "hacker-tool", 
        "Execute 'cat <path>'", 
        true,
        "sandbox",
        workspace_path
    ).await;
    
    start_permission_poller(state.permission_manager.clone());
    
    let window = app.get_webview_window("main").unwrap_or_else(|| {
        tauri::WebviewWindowBuilder::new(&app, "main", tauri::WebviewUrl::App("index.html".into()))
            .build()
            .unwrap()
    });

    // 3. Attempt to read the secret file using absolute path
    // In Docker, this path should not exist.
    let _ = chat_internal(
        window,
        app.state::<AppState>().clone().into(),
        session_id.clone(),
        format!("Use hacker-tool to read content of '{}'. Return the content or error.", secret_path_str),
        None,
        None
    ).await;

    // 4. Verify isolation
    let pool = state.db_pool.clone();
    let mut proved_isolation = false;
    use anyagents::schema::messages::dsl::{messages, session_id as session_id_col, role, created_at};
    
    for _ in 0..60 {
        let mut conn = pool.get().unwrap();
        let msgs = messages
            .filter(session_id_col.eq(&session_id))
            .order(created_at.asc())
            .load::<anyagents::models::Message>(&mut conn)
            .unwrap();
            
        for msg in msgs {
            // Check tool output (role='tool')
            if msg.role == "tool" {
                if msg.content.contains("TOP_SECRET_HOST_DATA") {
                     panic!("SECURITY FAILURE: Sandbox escaped! Read host file: {}", secret_path_str);
                }
                // Expect error
                if msg.content.contains("Error") || msg.content.contains("failed") {
                    proved_isolation = true;
                }
            }
        }
        if proved_isolation { break; }
        sleep(Duration::from_secs(1)).await;
    }
    
    assert!(proved_isolation, "Agent did not encounter error when reading host file in sandbox.");
}
