//! Shared helper functions for office workflow integration tests
//!
//! Provides utilities for:
//! - Creating test agents with realistic roles
//! - Simulating agent-to-agent communication
//! - Waiting for background mail processing
//! - Asserting mail thread properties

use anyagents::database::DbPool;
use anyagents::models::{Agent, NewAgent};
use anyagents::schema::agents;
use anycowork::commands::mail::{get_mail_threads, MailThreadWithPreview};
use anycowork::AppState;
use diesel::prelude::*;
use std::sync::Arc;
use std::time::Duration;
use tauri::test::mock_builder;
use tauri::Manager;

/// Create a test agent with specified name, avatar, and description
pub fn create_test_agent(
    pool: &DbPool,
    name: &str,
    avatar: &str,
    description: &str,
) -> Agent {
    let ai_config_json = serde_json::json!({
        "provider": "openai",
        "model": "gpt-4o"
    })
    .to_string();

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
        system_prompt: Some(format!(
            "You are {}. {}",
            name, description
        )),
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

    agents::table
        .filter(agents::id.eq(&new_agent.id))
        .first::<Agent>(&mut conn)
        .expect("Failed to retrieve inserted agent")
}

/// Create a test AppState for integration tests
pub fn create_test_app_state() -> AppState {
    let pool = anyagents::database::create_test_pool();
    AppState {
        db_pool: pool.clone(),
        pending_approvals: Arc::new(dashmap::DashMap::new()),
        telegram_manager: Arc::new(anycowork::telegram::TelegramBotManager::new(
            pool.clone(),
        )),
        permission_manager: Arc::new(anyagents::permissions::PermissionManager::new()),
    }
}

/// Create a Tauri app with test state
pub fn create_test_app(state: AppState) -> tauri::App<tauri::test::MockRuntime> {
    let app = mock_builder()
        .build(tauri::generate_context!())
        .expect("error while running tauri application");
    app.manage(state);
    app
}

/// Wait for background mail processing to complete
///
/// Polls the recipient's inbox until expected_count emails are received or timeout occurs.
pub async fn wait_for_mail_processing(
    state: tauri::State<'_, AppState>,
    recipient_id: &str,
    expected_count: usize,
    timeout_secs: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let start = std::time::Instant::now();
    let timeout = Duration::from_secs(timeout_secs);

    loop {
        if start.elapsed() > timeout {
            return Err(format!(
                "Timeout waiting for {} emails to arrive (waited {}s)",
                expected_count, timeout_secs
            )
            .into());
        }

        let inbox = get_mail_threads(
            state.clone(),
            Some(recipient_id.to_string()),
            Some("inbox".to_string()),
            None,
        )
        .await?;

        if inbox.len() >= expected_count {
            return Ok(());
        }

        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

/// Assert that a thread with the given subject exists in the threads list
pub fn assert_thread_has_subject<'a>(
    threads: &'a [MailThreadWithPreview],
    subject_contains: &str,
) -> &'a MailThreadWithPreview {
    threads
        .iter()
        .find(|t| t.subject.contains(subject_contains))
        .unwrap_or_else(|| {
            panic!(
                "Expected thread with subject containing '{}', but found threads: {:?}",
                subject_contains,
                threads.iter().map(|t| &t.subject).collect::<Vec<_>>()
            )
        })
}

/// Assert that the recipient received an email from the specified sender
pub async fn assert_received_from(
    state: tauri::State<'_, AppState>,
    recipient_id: &str,
    sender_name_contains: &str,
) -> Result<MailThreadWithPreview, Box<dyn std::error::Error>> {
    let inbox = get_mail_threads(
        state.clone(),
        Some(recipient_id.to_string()),
        Some("inbox".to_string()),
        None,
    )
    .await?;

    inbox
        .iter()
        .find(|thread| {
            thread
                .last_sender_name
                .as_ref()
                .map(|name| name.contains(sender_name_contains))
                .unwrap_or(false)
        })
        .cloned()
        .ok_or_else(|| {
            format!(
                "No email from sender containing '{}' found in inbox",
                sender_name_contains
            )
            .into()
        })
}

/// Get the sent folder for an agent
pub async fn get_sent_folder(
    state: tauri::State<'_, AppState>,
    agent_id: &str,
) -> Result<Vec<MailThreadWithPreview>, String> {
    get_mail_threads(
        state.clone(),
        Some(agent_id.to_string()),
        Some("sent".to_string()),
        None,
    )
    .await
}

/// Get the inbox for an agent
pub async fn get_inbox(
    state: tauri::State<'_, AppState>,
    agent_id: &str,
) -> Result<Vec<MailThreadWithPreview>, String> {
    get_mail_threads(
        state.clone(),
        Some(agent_id.to_string()),
        Some("inbox".to_string()),
        None,
    )
    .await
}
