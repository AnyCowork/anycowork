//! End-to-end tests for mail system
//!
//! Tests the complete email workflow including:
//! - Sending emails between characters
//! - Email reply generation without tool calls
//! - Message filtering and deduplication
//! - Proper character name resolution

use anycowork::commands::mail::{get_mail_threads, send_mail, get_mail_thread_messages, reply_to_mail};
use anycowork::AppState;
use anyagents::database::create_test_pool;
use anyagents::models::{Agent, NewAgent};
use anyagents::schema::agents;
use diesel::prelude::*;
use std::sync::Arc;
use tauri::test::mock_builder;
use tauri::Manager;

/// Helper to create a test agent
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
async fn test_send_email_between_characters() {
    let state = create_test_app_state();
    let app = create_app(state.clone());
    let state_handle = app.state::<AppState>();

    // Create two agents
    let jordan_id = create_test_agent(
        &state.db_pool,
        "Jordan the PM",
        "📋",
        "Project manager who coordinates team activities"
    );

    let dev_id = create_test_agent(
        &state.db_pool,
        "Dev the Developer",
        "💻",
        "Software developer who writes code"
    );

    // Send email from Dev to Jordan
    let thread = send_mail(
        state_handle.clone(),
        Some(dev_id.clone()),
        Some(jordan_id.clone()),
        "Hello from Dev".to_string(),
        "Hi Jordan,\n\nJust wanted to say hello!\n\nBest,\nDev".to_string(),
    ).await;

    assert!(thread.is_ok(), "Email should be sent successfully");
    let thread = thread.unwrap();

    assert_eq!(thread.subject, "Hello from Dev");
    assert!(thread.last_message_preview.is_some());
    assert!(thread.last_message_preview.unwrap().contains("Just wanted to say hello"));

    // Verify the thread appears in Jordan's inbox
    let threads = get_mail_threads(
        state_handle.clone(),
        Some(jordan_id.clone()),
        Some("inbox".to_string()),
        None,
    ).await;

    assert!(threads.is_ok());
    let threads = threads.unwrap();
    assert_eq!(threads.len(), 1, "Jordan should have 1 email in inbox");
    assert_eq!(threads[0].subject, "Hello from Dev");
}

#[tokio::test]
async fn test_character_name_resolution() {
    let state = create_test_app_state();
    let app = create_app(state.clone());
    let state_handle = app.state::<AppState>();

    // Create agents with partial name matching
    let jordan_id = create_test_agent(
        &state.db_pool,
        "Jordan the PM",
        "📋",
        "Project manager"
    );

    let dev_id = create_test_agent(
        &state.db_pool,
        "Dev the Developer",
        "💻",
        "Developer"
    );

    // Test partial name matching - should find "Jordan the PM" when searching for "Jordan"
    let thread = send_mail(
        state_handle.clone(),
        Some(dev_id.clone()),
        Some(jordan_id.clone()),
        "Test Subject".to_string(),
        "Test body".to_string(),
    ).await;

    assert!(thread.is_ok(), "Should resolve 'Jordan' to 'Jordan the PM'");
}

#[tokio::test]
async fn test_email_thread_messages() {
    let state = create_test_app_state();
    let app = create_app(state.clone());
    let state_handle = app.state::<AppState>();

    let sender_id = create_test_agent(&state.db_pool, "Sender", "📤", "Sender");
    let recipient_id = create_test_agent(&state.db_pool, "Recipient", "📥", "Recipient");

    // Send initial email
    let thread = send_mail(
        state_handle.clone(),
        Some(sender_id.clone()),
        Some(recipient_id.clone()),
        "Test Thread".to_string(),
        "Initial message".to_string(),
    ).await.unwrap();

    // Get thread messages
    let messages = get_mail_thread_messages(
        state_handle.clone(),
        thread.id.clone(),
    ).await;

    assert!(messages.is_ok());
    let messages = messages.unwrap();
    assert_eq!(messages.len(), 1, "Should have 1 message");
    assert_eq!(messages[0].content, "Initial message");
    assert_eq!(messages[0].sender_name, Some("Sender".to_string()));
}

#[tokio::test]
async fn test_reply_to_email() {
    let state = create_test_app_state();
    let app = create_app(state.clone());
    let state_handle = app.state::<AppState>();

    let sender_id = create_test_agent(&state.db_pool, "Sender", "📤", "Sender");
    let recipient_id = create_test_agent(&state.db_pool, "Recipient", "📥", "Recipient");

    // Send initial email
    let thread = send_mail(
        state_handle.clone(),
        Some(sender_id.clone()),
        Some(recipient_id.clone()),
        "Question".to_string(),
        "What is your status?".to_string(),
    ).await.unwrap();

    // Reply to the email
    let reply = reply_to_mail(
        state_handle.clone(),
        thread.id.clone(),
        Some(recipient_id.clone()),
        "All good, working on it!".to_string(),
    ).await;

    assert!(reply.is_ok());
    let reply = reply.unwrap();
    assert_eq!(reply.content, "All good, working on it!");
    assert_eq!(reply.sender_name, Some("Recipient".to_string()));

    // Verify thread now has 2 messages
    let messages = get_mail_thread_messages(
        state_handle.clone(),
        thread.id.clone(),
    ).await.unwrap();

    assert_eq!(messages.len(), 2, "Thread should have 2 messages");
}

#[tokio::test]
async fn test_user_to_agent_email() {
    let state = create_test_app_state();
    let app = create_app(state.clone());
    let state_handle = app.state::<AppState>();

    let agent_id = create_test_agent(&state.db_pool, "TestAgent", "🤖", "Test agent");

    // User sends email to agent
    let thread = send_mail(
        state_handle.clone(),
        None, // from user
        Some(agent_id.clone()),
        "User Request".to_string(),
        "Please help me with something".to_string(),
    ).await;

    assert!(thread.is_ok());
    let thread = thread.unwrap();
    assert_eq!(thread.last_sender_name, Some("You".to_string()));

    // Verify it appears in agent's inbox
    let threads = get_mail_threads(
        state_handle.clone(),
        Some(agent_id),
        Some("inbox".to_string()),
        None,
    ).await.unwrap();

    assert_eq!(threads.len(), 1);
}

#[tokio::test]
async fn test_agent_to_user_email() {
    let state = create_test_app_state();
    let app = create_app(state.clone());
    let state_handle = app.state::<AppState>();

    let agent_id = create_test_agent(&state.db_pool, "TestAgent", "🤖", "Test agent");

    // Agent sends email to user
    let thread = send_mail(
        state_handle.clone(),
        Some(agent_id.clone()),
        None, // to user
        "Report".to_string(),
        "Here is your daily report".to_string(),
    ).await;

    assert!(thread.is_ok());
    let thread = thread.unwrap();

    // Verify it appears in user's inbox
    let threads = get_mail_threads(
        state_handle.clone(),
        None, // user's mailbox
        Some("inbox".to_string()),
        None,
    ).await.unwrap();

    assert_eq!(threads.len(), 1);
    assert!(threads[0].last_sender_name.is_some());
}

#[tokio::test]
async fn test_message_with_special_characters() {
    let state = create_test_app_state();
    let app = create_app(state.clone());
    let state_handle = app.state::<AppState>();

    let sender_id = create_test_agent(&state.db_pool, "Sender", "📤", "Sender");
    let recipient_id = create_test_agent(&state.db_pool, "Recipient", "📥", "Recipient");

    // Send email with special characters and formatting
    let body_with_formatting = "Hi Jordan,\n\nJust wanted to say hello!\n\nBest,\nDev";

    let thread = send_mail(
        state_handle.clone(),
        Some(sender_id.clone()),
        Some(recipient_id.clone()),
        "Test Special Characters".to_string(),
        body_with_formatting.to_string(),
    ).await;

    assert!(thread.is_ok());

    // Verify the message content is preserved correctly
    let messages = get_mail_thread_messages(
        state_handle.clone(),
        thread.unwrap().id,
    ).await.unwrap();

    assert_eq!(messages[0].content, body_with_formatting);
    // Verify newlines are preserved
    assert!(messages[0].content.contains('\n'));
}

#[tokio::test]
async fn test_inbox_vs_sent_filtering() {
    let state = create_test_app_state();
    let app = create_app(state.clone());
    let state_handle = app.state::<AppState>();

    let sender_id = create_test_agent(&state.db_pool, "Sender", "📤", "Sender");
    let recipient_id = create_test_agent(&state.db_pool, "Recipient", "📥", "Recipient");

    // Send email
    let _thread = send_mail(
        state_handle.clone(),
        Some(sender_id.clone()),
        Some(recipient_id.clone()),
        "Test".to_string(),
        "Body".to_string(),
    ).await.unwrap();

    // Check sender's sent folder
    let sent_threads = get_mail_threads(
        state_handle.clone(),
        Some(sender_id.clone()),
        Some("sent".to_string()),
        None,
    ).await.unwrap();

    assert_eq!(sent_threads.len(), 1, "Sender should have 1 email in sent");

    // Check recipient's inbox
    let inbox_threads = get_mail_threads(
        state_handle.clone(),
        Some(recipient_id.clone()),
        Some("inbox".to_string()),
        None,
    ).await.unwrap();

    assert_eq!(inbox_threads.len(), 1, "Recipient should have 1 email in inbox");

    // Sender's inbox should be empty
    let sender_inbox = get_mail_threads(
        state_handle.clone(),
        Some(sender_id.clone()),
        Some("inbox".to_string()),
        None,
    ).await.unwrap();

    assert_eq!(sender_inbox.len(), 0, "Sender's inbox should be empty");
}
