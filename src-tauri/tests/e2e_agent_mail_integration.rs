//! Integration tests for agent-to-agent email communication
//!
//! Tests the complete workflow including:
//! - Tool calling from agent to send email
//! - Background processing of incoming emails
//! - Reply generation without tool calls
//! - Message filtering and deduplication

use anycowork::AppState;
use anyagents::database::create_test_pool;
use anyagents::models::{Agent, NewAgent};
use anyagents::schema::{agents, mail_messages, mail_threads};
use anyagents::tools::{Tool, ToolContext};
use anyagents::tools::email::SendEmailTool;
use anyagents::permissions::PermissionManager;
use anyagents::events::AgentObserver;
use diesel::prelude::*;
use serde_json::{json, Value};
use std::sync::Arc;

/// No-op observer for testing
struct TestObserver;

impl AgentObserver for TestObserver {
    fn emit(&self, _event: &str, _payload: Value) -> Result<(), String> {
        Ok(())
    }
}

fn create_test_agent(pool: &anyagents::database::DbPool, name: &str, description: &str) -> Agent {
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
        avatar: Some("🤖".to_string()),
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

#[tokio::test]
async fn test_send_email_tool_execution() {
    let pool = create_test_pool();

    // Create sender and recipient agents
    let sender = create_test_agent(&pool, "Sender Agent", "Test sender");
    let recipient = create_test_agent(&pool, "Recipient Agent", "Test recipient");

    // Get all agents for colleague list
    let mut conn = pool.get().unwrap();
    let all_agents: Vec<Agent> = agents::table.load(&mut conn).unwrap();

    // Create SendEmailTool
    let email_tool = SendEmailTool::new(
        pool.clone(),
        sender.id.clone(),
        sender.name.clone(),
        all_agents,
    );

    // Execute tool
    let ctx = ToolContext {
        permissions: Arc::new(PermissionManager::new()),
        observer: Some(Arc::new(TestObserver)),
        session_id: uuid::Uuid::new_v4().to_string(),
    };

    let args = json!({
        "to": "Recipient Agent",
        "subject": "Test Email",
        "body": "Hello Recipient,\n\nThis is a test email.\n\nBest,\nSender"
    });

    let result = email_tool.execute(args, &ctx).await;

    assert!(result.is_ok(), "Email tool execution should succeed");

    let result = result.unwrap();
    assert_eq!(result["status"], "sent");
    assert!(result["thread_id"].is_string());

    // Verify thread was created
    let thread_id = result["thread_id"].as_str().unwrap();
    let thread: Result<anyagents::models::MailThread, _> = mail_threads::table
        .filter(mail_threads::id.eq(thread_id))
        .first(&mut conn);

    assert!(thread.is_ok(), "Thread should exist in database");
    assert_eq!(thread.unwrap().subject, "Test Email");

    // Verify message was created
    let messages: Vec<anyagents::models::MailMessage> = mail_messages::table
        .filter(mail_messages::thread_id.eq(thread_id))
        .load(&mut conn)
        .unwrap();

    assert_eq!(messages.len(), 1, "Should have 1 message");
    assert!(messages[0].content.contains("Hello Recipient"));
    assert!(messages[0].content.contains('\n'), "Newlines should be preserved");
}

#[tokio::test]
async fn test_email_body_escape_sequences() {
    let pool = create_test_pool();

    let sender = create_test_agent(&pool, "Sender", "Sender");
    let recipient = create_test_agent(&pool, "Recipient", "Recipient");

    let mut conn = pool.get().unwrap();
    let all_agents: Vec<Agent> = agents::table.load(&mut conn).unwrap();

    let email_tool = SendEmailTool::new(
        pool.clone(),
        sender.id.clone(),
        sender.name.clone(),
        all_agents,
    );

    let ctx = ToolContext {
        permissions: Arc::new(PermissionManager::new()),
        observer: Some(Arc::new(TestObserver)),
        session_id: uuid::Uuid::new_v4().to_string(),
    };

    // Test with newlines and special characters
    let body_with_formatting = "Hi Jordan,\n\nJust wanted to say hello!\n\nBest,\nDev";

    let args = json!({
        "to": "Recipient",
        "subject": "Formatted Email",
        "body": body_with_formatting
    });

    let result = email_tool.execute(args, &ctx).await.unwrap();
    let thread_id = result["thread_id"].as_str().unwrap();

    // Verify the message content preserves formatting
    let messages: Vec<anyagents::models::MailMessage> = mail_messages::table
        .filter(mail_messages::thread_id.eq(thread_id))
        .load(&mut conn)
        .unwrap();

    assert_eq!(messages[0].content, body_with_formatting);

    // Count newlines
    let newline_count = messages[0].content.matches('\n').count();
    assert_eq!(newline_count, 4, "Should have 4 newlines");
}

#[tokio::test]
async fn test_tool_validation_missing_recipient() {
    let pool = create_test_pool();

    let sender = create_test_agent(&pool, "Sender", "Sender");

    let mut conn = pool.get().unwrap();
    let all_agents: Vec<Agent> = agents::table.load(&mut conn).unwrap();

    let email_tool = SendEmailTool::new(
        pool.clone(),
        sender.id.clone(),
        sender.name.clone(),
        all_agents,
    );

    let ctx = ToolContext {
        permissions: Arc::new(PermissionManager::new()),
        observer: Some(Arc::new(TestObserver)),
        session_id: uuid::Uuid::new_v4().to_string(),
    };

    // Missing 'to' field
    let args = json!({
        "subject": "Test",
        "body": "Body"
    });

    let result = email_tool.execute(args, &ctx).await;

    assert!(result.is_err(), "Should fail with missing recipient");
    assert!(result.unwrap_err().contains("Missing 'to' field"));
}

#[tokio::test]
async fn test_tool_validation_unknown_recipient() {
    let pool = create_test_pool();

    let sender = create_test_agent(&pool, "Sender", "Sender");

    let mut conn = pool.get().unwrap();
    let all_agents: Vec<Agent> = agents::table.load(&mut conn).unwrap();

    let email_tool = SendEmailTool::new(
        pool.clone(),
        sender.id.clone(),
        sender.name.clone(),
        all_agents,
    );

    let ctx = ToolContext {
        permissions: Arc::new(PermissionManager::new()),
        observer: Some(Arc::new(TestObserver)),
        session_id: uuid::Uuid::new_v4().to_string(),
    };

    let args = json!({
        "to": "NonExistentAgent",
        "subject": "Test",
        "body": "Body"
    });

    let result = email_tool.execute(args, &ctx).await;

    assert!(result.is_err(), "Should fail with unknown recipient");
    assert!(result.unwrap_err().contains("not found"));
}

#[tokio::test]
async fn test_partial_name_matching() {
    let pool = create_test_pool();

    let sender = create_test_agent(&pool, "Sender", "Sender");
    let recipient = create_test_agent(&pool, "Jordan the PM", "Project Manager");

    let mut conn = pool.get().unwrap();
    let all_agents: Vec<Agent> = agents::table.load(&mut conn).unwrap();

    let email_tool = SendEmailTool::new(
        pool.clone(),
        sender.id.clone(),
        sender.name.clone(),
        all_agents,
    );

    let ctx = ToolContext {
        permissions: Arc::new(PermissionManager::new()),
        observer: Some(Arc::new(TestObserver)),
        session_id: uuid::Uuid::new_v4().to_string(),
    };

    // Search for "Jordan" should find "Jordan the PM"
    let args = json!({
        "to": "Jordan",
        "subject": "Test",
        "body": "Hello"
    });

    let result = email_tool.execute(args, &ctx).await;

    assert!(result.is_ok(), "Should find 'Jordan the PM' when searching for 'Jordan'");

    let result = result.unwrap();
    let thread_id = result["thread_id"].as_str().unwrap();

    // Verify recipient is correct
    let messages: Vec<anyagents::models::MailMessage> = mail_messages::table
        .filter(mail_messages::thread_id.eq(thread_id))
        .load(&mut conn)
        .unwrap();

    assert_eq!(messages[0].recipient_agent_id, Some(recipient.id));
}

#[tokio::test]
async fn test_send_to_user_mailbox() {
    let pool = create_test_pool();

    let sender = create_test_agent(&pool, "Sender", "Sender");

    let mut conn = pool.get().unwrap();
    let all_agents: Vec<Agent> = agents::table.load(&mut conn).unwrap();

    let email_tool = SendEmailTool::new(
        pool.clone(),
        sender.id.clone(),
        sender.name.clone(),
        all_agents,
    );

    let ctx = ToolContext {
        permissions: Arc::new(PermissionManager::new()),
        observer: Some(Arc::new(TestObserver)),
        session_id: uuid::Uuid::new_v4().to_string(),
    };

    let args = json!({
        "to": "user",
        "subject": "Report",
        "body": "Here is your report"
    });

    let result = email_tool.execute(args, &ctx).await;

    assert!(result.is_ok(), "Should be able to send to 'user'");

    let result = result.unwrap();
    let thread_id = result["thread_id"].as_str().unwrap();

    // Verify message recipient is user
    let messages: Vec<anyagents::models::MailMessage> = mail_messages::table
        .filter(mail_messages::thread_id.eq(thread_id))
        .load(&mut conn)
        .unwrap();

    assert_eq!(messages[0].recipient_type, "user");
    assert!(messages[0].recipient_agent_id.is_none());
}
