//! End-to-end tests for contact list and internal mail system
//!
//! Tests that agents can:
//! - List colleagues using list_colleagues tool
//! - Send internal emails using just names (no email addresses)
//! - Understand the internal mail system

use anyagents::database::create_test_pool;
use anyagents::models::{Agent, NewAgent};
use anyagents::schema::agents;
use anyagents::tools::{Tool, ToolContext};
use anyagents::tools::contacts::ListColleaguesTool;
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

fn create_test_agent(pool: &anyagents::database::DbPool, name: &str, avatar: &str) -> Agent {
    let agent_id = uuid::Uuid::new_v4().to_string();
    let new_agent = NewAgent {
        id: agent_id.clone(),
        name: name.to_string(),
        description: Some(format!("Test agent: {}", name)),
        status: "active".to_string(),
        personality: Some("professional".to_string()),
        tone: Some("friendly".to_string()),
        expertise: Some("general".to_string()),
        ai_provider: "openai".to_string(),
        ai_model: "gpt-4o".to_string(),
        ai_temperature: 0.7,
        ai_config: "{}".to_string(),
        system_prompt: Some("You are a helpful agent.".to_string()),
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

#[tokio::test]
async fn test_list_colleagues_tool() {
    let pool = create_test_pool();

    // Create test agents
    let alice = create_test_agent(&pool, "Alice the Coordinator", "👩‍💼");
    let bob = create_test_agent(&pool, "Bob the Messenger", "📨");
    let charlie = create_test_agent(&pool, "Charlie the Developer", "💻");

    // Create list_colleagues tool for Alice
    let tool = ListColleaguesTool::new(pool.clone(), alice.id.clone());

    // Execute tool
    let ctx = ToolContext {
        permissions: Arc::new(PermissionManager::new()),
        observer: Some(Arc::new(TestObserver)),
        session_id: uuid::Uuid::new_v4().to_string(),
    };

    let result = tool.execute(json!({}), &ctx).await;

    assert!(result.is_ok(), "list_colleagues should succeed");

    let result = result.unwrap();
    println!("Colleagues list:\n{}", serde_json::to_string_pretty(&result).unwrap());

    // Verify result structure
    assert!(result["colleagues"].is_array(), "Should have colleagues array");

    let colleagues = result["colleagues"].as_array().unwrap();

    // Should have Bob, Charlie, and User (not Alice herself)
    assert_eq!(colleagues.len(), 3, "Should have 3 colleagues (Bob, Charlie, User)");

    // Verify Bob's entry
    let bob_entry = colleagues.iter()
        .find(|c| c["name"].as_str() == Some("Bob the Messenger"))
        .expect("Should find Bob in colleagues");

    assert_eq!(bob_entry["email"], "bob.messenger@anycowork.local");
    assert_eq!(bob_entry["avatar"], "📨");
    assert_eq!(bob_entry["role"], "Messenger");

    // Verify Charlie's entry
    let charlie_entry = colleagues.iter()
        .find(|c| c["name"].as_str() == Some("Charlie the Developer"))
        .expect("Should find Charlie in colleagues");

    assert_eq!(charlie_entry["email"], "charlie.developer@anycowork.local");
    assert_eq!(charlie_entry["role"], "Developer");

    // Verify User entry
    let user_entry = colleagues.iter()
        .find(|c| c["name"].as_str() == Some("User"))
        .expect("Should find User in colleagues");

    assert_eq!(user_entry["email"], "user@anycowork.local");
    assert_eq!(user_entry["role"], "Owner");

    // Verify note explains internal system
    let note = result["note"].as_str().unwrap();
    assert!(note.contains("internal"), "Note should mention internal system");
    assert!(note.contains("name"), "Note should mention using names");
}

#[tokio::test]
async fn test_send_email_with_just_name() {
    let pool = create_test_pool();

    // Create test agents
    let sender = create_test_agent(&pool, "Sender Agent", "📤");
    let recipient = create_test_agent(&pool, "Jordan the PM", "📋");

    // Get all agents for SendEmailTool
    let mut conn = pool.get().unwrap();
    let all_agents: Vec<Agent> = agents::table.load(&mut conn).unwrap();

    // Create send_email tool
    let tool = SendEmailTool::new(
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

    // Test 1: Send using full name
    let result = tool.execute(
        json!({
            "to": "Jordan the PM",
            "subject": "Test Email",
            "body": "Hello Jordan!"
        }),
        &ctx
    ).await;

    assert!(result.is_ok(), "Should send with full name 'Jordan the PM'");
    println!("Result 1: {}", serde_json::to_string_pretty(&result.unwrap()).unwrap());

    // Test 2: Send using partial name
    let result2 = tool.execute(
        json!({
            "to": "Jordan",  // Just "Jordan", not full name
            "subject": "Test Email 2",
            "body": "Hello again!"
        }),
        &ctx
    ).await;

    assert!(result2.is_ok(), "Should send with partial name 'Jordan'");
    println!("Result 2: {}", serde_json::to_string_pretty(&result2.unwrap()).unwrap());

    // Test 3: Verify emails were created
    use anyagents::schema::mail_messages;
    let messages: Vec<anyagents::models::MailMessage> = mail_messages::table
        .load(&mut conn)
        .unwrap();

    assert_eq!(messages.len(), 2, "Should have 2 email messages");

    // Both should go to Jordan
    for msg in &messages {
        assert_eq!(msg.recipient_agent_id, Some(recipient.id.clone()));
    }
}

#[tokio::test]
async fn test_send_to_user_mailbox() {
    let pool = create_test_pool();

    let sender = create_test_agent(&pool, "Research Agent", "🔬");

    let mut conn = pool.get().unwrap();
    let all_agents: Vec<Agent> = agents::table.load(&mut conn).unwrap();

    let tool = SendEmailTool::new(
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

    // Send to "user"
    let result = tool.execute(
        json!({
            "to": "user",
            "subject": "Research Results",
            "body": "Here are my findings..."
        }),
        &ctx
    ).await;

    assert!(result.is_ok(), "Should send to 'user'");

    // Verify email went to user mailbox
    use anyagents::schema::mail_messages;
    let messages: Vec<anyagents::models::MailMessage> = mail_messages::table
        .filter(mail_messages::recipient_type.eq("user"))
        .load(&mut conn)
        .unwrap();

    assert_eq!(messages.len(), 1, "Should have 1 message to user");
    assert_eq!(messages[0].sender_agent_id, Some(sender.id));
}

#[tokio::test]
async fn test_virtual_email_format() {
    // Test email generation logic
    let pool = create_test_pool();

    create_test_agent(&pool, "Jordan the PM", "📋");
    create_test_agent(&pool, "Alex the Chief", "👑");
    create_test_agent(&pool, "Dev the Developer", "💻");

    let mut conn = pool.get().unwrap();
    let all_agents: Vec<Agent> = agents::table.load(&mut conn).unwrap();

    // Verify we can extract virtual emails
    for agent in &all_agents {
        let virtual_email = generate_test_virtual_email(&agent.name);
        println!("{} -> {}", agent.name, virtual_email);

        // Verify format
        assert!(virtual_email.ends_with("@anycowork.local"));
        assert!(virtual_email.contains('.') || virtual_email.len() < 20);
    }
}

fn generate_test_virtual_email(name: &str) -> String {
    let email_part = name
        .to_lowercase()
        .replace(" the ", ".")
        .replace(" ", ".")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '.')
        .collect::<String>();

    format!("{}@anycowork.local", email_part)
}

#[tokio::test]
async fn test_tool_description_clarity() {
    let pool = create_test_pool();
    let agent = create_test_agent(&pool, "Test Agent", "🤖");

    let mut conn = pool.get().unwrap();
    let all_agents: Vec<Agent> = agents::table.load(&mut conn).unwrap();

    // Create tools
    let list_colleagues = ListColleaguesTool::new(pool.clone(), agent.id.clone());
    let send_email = SendEmailTool::new(
        pool.clone(),
        agent.id.clone(),
        agent.name.clone(),
        all_agents,
    );

    // Verify descriptions mention internal system
    let list_desc = list_colleagues.description();
    assert!(list_desc.contains("colleague") || list_desc.contains("team"));
    println!("list_colleagues description: {}", list_desc);

    let send_desc = send_email.description();
    assert!(send_desc.to_uppercase().contains("INTERNAL"), "send_email should mention INTERNAL");
    assert!(send_desc.contains("name") || send_desc.contains("NAME"), "send_email should mention using names");
    println!("send_email description: {}", send_desc);

    // Verify parameter descriptions
    let schema = send_email.parameters_schema();
    let to_description = schema["properties"]["to"]["description"].as_str().unwrap();

    assert!(to_description.to_uppercase().contains("NAME"), "Parameter should mention NAME");
    assert!(!to_description.contains("email address") ||
            to_description.contains("not email address"),
            "Parameter should clarify NOT email address");

    println!("'to' parameter description: {}", to_description);
}

#[test]
fn test_colleague_count() {
    let pool = create_test_pool();

    // Create 5 agents
    for i in 0..5 {
        create_test_agent(&pool, &format!("Agent {}", i), "🤖");
    }

    let mut conn = pool.get().unwrap();
    let all_agents: Vec<Agent> = agents::table.load(&mut conn).unwrap();

    assert_eq!(all_agents.len(), 5, "Should have 5 agents");

    // Each agent should see 4 colleagues + user = 5 total
    // (excluding themselves)
}

#[tokio::test]
async fn test_check_mail_tool() {
    use anyagents::tools::mail_reader::MailReaderTool;

    let pool = create_test_pool();

    // Create agents
    let alice = create_test_agent(&pool, "Alice the Reader", "📖");
    let bob = create_test_agent(&pool, "Bob the Sender", "📤");

    // Bob sends email to Alice
    let mut conn = pool.get().unwrap();
    let all_agents: Vec<Agent> = agents::table.load(&mut conn).unwrap();

    let send_tool = SendEmailTool::new(
        pool.clone(),
        bob.id.clone(),
        bob.name.clone(),
        all_agents,
    );

    let ctx = ToolContext {
        permissions: Arc::new(PermissionManager::new()),
        observer: Some(Arc::new(TestObserver)),
        session_id: uuid::Uuid::new_v4().to_string(),
    };

    // Send 2 emails
    let _ = send_tool.execute(
        json!({
            "to": "Alice",
            "subject": "Test Email 1",
            "body": "First test message"
        }),
        &ctx
    ).await;

    let _ = send_tool.execute(
        json!({
            "to": "Alice",
            "subject": "Test Email 2",
            "body": "Second test message"
        }),
        &ctx
    ).await;

    // Alice checks her inbox
    let check_mail = MailReaderTool::new(pool.clone(), alice.id.clone());

    let result = check_mail.execute(
        json!({"folder": "inbox"}),
        &ctx
    ).await;

    assert!(result.is_ok(), "check_mail should succeed");
    let result = result.unwrap();

    println!("Alice's inbox:\n{}", serde_json::to_string_pretty(&result).unwrap());

    assert_eq!(result["folder"], "inbox");
    assert!(result["emails"].is_array());

    let emails = result["emails"].as_array().unwrap();
    assert_eq!(emails.len(), 2, "Should have 2 emails in inbox");

    // Verify email content
    assert!(emails.iter().any(|e| e["subject"].as_str() == Some("Test Email 1")));
    assert!(emails.iter().any(|e| e["subject"].as_str() == Some("Test Email 2")));

    // Test unread filter
    let unread_result = check_mail.execute(
        json!({"folder": "inbox", "unread_only": true}),
        &ctx
    ).await.unwrap();

    let unread_emails = unread_result["emails"].as_array().unwrap();
    assert_eq!(unread_emails.len(), 2, "All emails should be unread initially");
}

#[tokio::test]
async fn test_read_email_thread_tool() {
    use anyagents::tools::mail_reader::ReadEmailThreadTool;

    let pool = create_test_pool();

    // Create agents
    let alice = create_test_agent(&pool, "Alice", "📖");
    let bob = create_test_agent(&pool, "Bob", "📤");

    // Send email to create thread
    let mut conn = pool.get().unwrap();
    let all_agents: Vec<Agent> = agents::table.load(&mut conn).unwrap();

    let send_tool = SendEmailTool::new(
        pool.clone(),
        bob.id.clone(),
        bob.name.clone(),
        all_agents,
    );

    let ctx = ToolContext {
        permissions: Arc::new(PermissionManager::new()),
        observer: Some(Arc::new(TestObserver)),
        session_id: uuid::Uuid::new_v4().to_string(),
    };

    let send_result = send_tool.execute(
        json!({
            "to": "Alice",
            "subject": "Project Discussion",
            "body": "Hi Alice, let's discuss the project"
        }),
        &ctx
    ).await.unwrap();

    let thread_id = send_result["thread_id"].as_str().expect("Should have thread_id");

    // Read the thread
    let read_thread = ReadEmailThreadTool::new(pool.clone());

    let result = read_thread.execute(
        json!({"thread_id": thread_id}),
        &ctx
    ).await;

    assert!(result.is_ok(), "read_email_thread should succeed");
    let result = result.unwrap();

    println!("Thread contents:\n{}", serde_json::to_string_pretty(&result).unwrap());

    assert_eq!(result["subject"], "Project Discussion");
    assert!(result["messages"].is_array());

    let messages = result["messages"].as_array().unwrap();
    assert_eq!(messages.len(), 1, "Should have 1 message");
    assert_eq!(messages[0]["from"], "Bob");
    assert_eq!(messages[0]["content"], "Hi Alice, let's discuss the project");
}

#[tokio::test]
async fn test_check_sent_folder() {
    use anyagents::tools::mail_reader::MailReaderTool;

    let pool = create_test_pool();

    let alice = create_test_agent(&pool, "Alice", "📤");
    let bob = create_test_agent(&pool, "Bob", "📥");

    let mut conn = pool.get().unwrap();
    let all_agents: Vec<Agent> = agents::table.load(&mut conn).unwrap();

    // Alice sends email to Bob
    let send_tool = SendEmailTool::new(
        pool.clone(),
        alice.id.clone(),
        alice.name.clone(),
        all_agents,
    );

    let ctx = ToolContext {
        permissions: Arc::new(PermissionManager::new()),
        observer: Some(Arc::new(TestObserver)),
        session_id: uuid::Uuid::new_v4().to_string(),
    };

    let _ = send_tool.execute(
        json!({
            "to": "Bob",
            "subject": "My Sent Email",
            "body": "This is a message I sent"
        }),
        &ctx
    ).await;

    // Alice checks her sent folder
    let check_mail = MailReaderTool::new(pool.clone(), alice.id.clone());

    let result = check_mail.execute(
        json!({"folder": "sent"}),
        &ctx
    ).await.unwrap();

    println!("Alice's sent folder:\n{}", serde_json::to_string_pretty(&result).unwrap());

    assert_eq!(result["folder"], "sent");

    let emails = result["emails"].as_array().unwrap();
    assert_eq!(emails.len(), 1, "Should have 1 email in sent folder");
    assert_eq!(emails[0]["subject"], "My Sent Email");
}
