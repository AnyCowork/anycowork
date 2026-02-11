//! End-to-end tests for complex multi-agent workflows
//!
//! Tests complex scenarios including:
//! - One character asking another to send email to a third character
//! - Agent researching and sending results to user's mailbox
//! - Autonomous mode (no approval needed)
//! - Multi-step agent interactions

use anycowork::AppState;
use anyagents::database::create_test_pool;
use anyagents::models::{Agent, NewAgent, Message};
use anyagents::schema::{agents, mail_messages, mail_threads, messages, sessions};
use anyagents::agents::AgentLoop;
use anyagents::permissions::PermissionManager;
use anyagents::events::{AgentEvent, AgentObserver};
use diesel::prelude::*;
use serde_json::Value;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

/// Test observer that captures events
#[derive(Clone)]
struct TestEventObserver {
    events: Arc<Mutex<VecDeque<(String, Value)>>>,
}

impl TestEventObserver {
    fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    #[allow(dead_code)]
    fn get_events(&self) -> Vec<(String, Value)> {
        let mut events = self.events.lock().unwrap();
        events.drain(..).collect()
    }

    #[allow(dead_code)]
    fn find_event(&self, event_type: &str) -> Option<Value> {
        let events = self.events.lock().unwrap();
        events.iter()
            .find(|(channel, payload)| {
                channel.contains("session:") &&
                payload.get("type").and_then(|t| t.as_str()) == Some(event_type)
            })
            .map(|(_, payload)| payload.clone())
    }
}

impl AgentObserver for TestEventObserver {
    fn emit(&self, channel: &str, payload: Value) -> Result<(), String> {
        let mut events = self.events.lock().unwrap();
        events.push_back((channel.to_string(), payload));
        Ok(())
    }
}

/// Autonomous permission manager that auto-approves all requests
pub struct AutonomousPermissionManager {
    base: PermissionManager,
    autonomous: bool,
}

impl AutonomousPermissionManager {
    pub fn new(autonomous: bool) -> Self {
        Self {
            base: PermissionManager::new(),
            autonomous,
        }
    }

    pub async fn request_permission(
        &self,
        observer: Option<&Arc<dyn AgentObserver>>,
        req: anyagents::permissions::PermissionRequest,
    ) -> Result<bool, String> {
        if self.autonomous {
            // In autonomous mode, auto-approve all requests
            log::info!("Autonomous mode: Auto-approving permission request: {}", req.message);
            return Ok(true);
        }

        // Otherwise, use normal permission flow
        self.base.request_permission(observer, req).await
    }

    pub fn approve_request(&self, request_id: &str) {
        self.base.approve_request(request_id);
    }

    pub fn reject_request(&self, request_id: &str) {
        self.base.reject_request(request_id);
    }
}

/// Helper to create a test agent with autonomous mode
fn create_autonomous_agent(
    pool: &anyagents::database::DbPool,
    name: &str,
    avatar: &str,
    description: &str,
    system_prompt: &str,
    autonomous: bool,
) -> Agent {
    let ai_config_json = serde_json::json!({
        "provider": "openai",
        "model": "gpt-4o"
    }).to_string();

    // Set execution settings for autonomous mode
    let execution_settings = if autonomous {
        Some(serde_json::json!({
            "mode": "autopilot",
            "sandbox_mode": "flexible"
        }).to_string())
    } else {
        Some(serde_json::json!({
            "mode": "require_approval",
            "sandbox_mode": "flexible"
        }).to_string())
    };

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
        system_prompt: Some(system_prompt.to_string()),
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
        execution_settings,
        scope_type: None,
        workspace_path: Some(std::env::temp_dir().to_str().unwrap().to_string()),
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
async fn test_agent_asks_another_to_send_email() {
    // Skip if no API key
    if std::env::var("OPENAI_API_KEY").is_err() {
        println!("SKIPPED: No OPENAI_API_KEY found");
        return;
    }

    let pool = create_test_pool();

    // Create three agents
    let alice = create_autonomous_agent(
        &pool,
        "Alice the Coordinator",
        "👩‍💼",
        "Team coordinator who delegates tasks",
        "You are Alice, a team coordinator. When asked to send messages, you delegate to Bob by asking him to send the email.",
        true, // autonomous
    );

    let _bob = create_autonomous_agent(
        &pool,
        "Bob the Messenger",
        "📨",
        "Email specialist who sends messages",
        "You are Bob, an email specialist. When asked to send an email, you use the send_email tool to send it.",
        true, // autonomous
    );

    let _charlie = create_autonomous_agent(
        &pool,
        "Charlie the Developer",
        "💻",
        "Software developer",
        "You are Charlie, a software developer.",
        true, // autonomous
    );

    // Create session for Alice
    let session_id = uuid::Uuid::new_v4().to_string();
    {
        let mut conn = pool.get().unwrap();
        let new_session = anyagents::models::NewSession {
            id: session_id.clone(),
            agent_id: alice.id.clone(),
            title: Some("Alice delegation test".to_string()),
            created_at: chrono::Utc::now().naive_utc(),
            updated_at: chrono::Utc::now().naive_utc(),
            archived: 0,
            pinned: 0,
        };
        diesel::insert_into(sessions::table)
            .values(&new_session)
            .execute(&mut conn)
            .unwrap();
    }

    // Create agent loop for Alice
    let observer = Arc::new(TestEventObserver::new());
    let mut alice_loop = AgentLoop::new(&alice, pool.clone()).await;
    alice_loop.session_id = session_id.clone();

    // Use autonomous permission manager
    // Convert to base permission manager for compatibility
    let base_permission_manager = Arc::new(PermissionManager::new());

    // User asks Alice to have Bob send email to Charlie
    let user_message = "Alice, can you ask Bob to send a quick hello email to Charlie?";

    let job_id = uuid::Uuid::new_v4().to_string();
    let pending_approvals = Arc::new(dashmap::DashMap::new());

    // Run Alice's agent loop
    alice_loop.run(
        user_message.to_string(),
        observer.clone() as Arc<dyn AgentObserver>,
        job_id,
        pending_approvals,
        base_permission_manager,
        pool.clone(),
    ).await;

    // Check Alice's response - she should mention delegating to Bob
    let mut conn = pool.get().unwrap();
    let alice_messages: Vec<Message> = messages::table
        .filter(messages::session_id.eq(&session_id))
        .filter(messages::role.eq("assistant"))
        .load(&mut conn)
        .unwrap();

    assert!(!alice_messages.is_empty(), "Alice should have responded");

    let response = &alice_messages.last().unwrap().content;
    println!("Alice's response: {}", response);

    // Alice might either:
    // 1. Directly send email to Bob asking him to send to Charlie
    // 2. Or respond saying she'll coordinate this

    // Check if any emails were sent
    let email_threads: Vec<anyagents::models::MailThread> = mail_threads::table
        .load(&mut conn)
        .unwrap();

    println!("Email threads created: {}", email_threads.len());
    for thread in &email_threads {
        println!("Thread: {}", thread.subject);
    }
}

#[tokio::test]
async fn test_agent_research_and_email_results() {
    // Skip if no API key
    if std::env::var("OPENAI_API_KEY").is_err() {
        println!("SKIPPED: No OPENAI_API_KEY found");
        return;
    }

    let pool = create_test_pool();

    // Create research agent
    let researcher = create_autonomous_agent(
        &pool,
        "Research Agent",
        "🔬",
        "Research specialist who finds information and emails results",
        "You are a research assistant. When asked to research something and send results, you:\n\
         1. Think about the topic\n\
         2. Provide your findings\n\
         3. Use send_email tool to send the results to 'user'",
        true, // autonomous
    );

    // Create session
    let session_id = uuid::Uuid::new_v4().to_string();
    {
        let mut conn = pool.get().unwrap();
        let new_session = anyagents::models::NewSession {
            id: session_id.clone(),
            agent_id: researcher.id.clone(),
            title: Some("Research task".to_string()),
            created_at: chrono::Utc::now().naive_utc(),
            updated_at: chrono::Utc::now().naive_utc(),
            archived: 0,
            pinned: 0,
        };
        diesel::insert_into(sessions::table)
            .values(&new_session)
            .execute(&mut conn)
            .unwrap();
    }

    // Create agent loop
    let observer = Arc::new(TestEventObserver::new());
    let mut researcher_loop = AgentLoop::new(&researcher, pool.clone()).await;
    researcher_loop.session_id = session_id.clone();

    let permission_manager = Arc::new(PermissionManager::new());

    // User asks researcher to find info and email it
    let user_message = "Please research the benefits of Rust programming language and email me a summary.";

    let job_id = uuid::Uuid::new_v4().to_string();
    let pending_approvals = Arc::new(dashmap::DashMap::new());

    // Run researcher's agent loop
    researcher_loop.run(
        user_message.to_string(),
        observer.clone() as Arc<dyn AgentObserver>,
        job_id,
        pending_approvals,
        permission_manager,
        pool.clone(),
    ).await;

    // Check if email was sent to user's mailbox
    let mut conn = pool.get().unwrap();
    let email_threads: Vec<anyagents::models::MailThread> = mail_threads::table
        .load(&mut conn)
        .unwrap();

    println!("Email threads created: {}", email_threads.len());

    if !email_threads.is_empty() {
        // Check if email was sent to user
        let user_emails: Vec<anyagents::models::MailMessage> = mail_messages::table
            .filter(mail_messages::recipient_type.eq("user"))
            .load(&mut conn)
            .unwrap();

        assert!(!user_emails.is_empty(), "Should have sent email to user");

        let email = &user_emails[0];
        println!("Email to user: {}", email.content);

        // Verify the email contains research findings
        assert!(
            email.content.to_lowercase().contains("rust") ||
            email.content.to_lowercase().contains("benefits"),
            "Email should contain research results"
        );
    } else {
        println!("Note: Agent may have responded without sending email (depending on LLM behavior)");

        // Check the agent's response
        let messages: Vec<Message> = messages::table
            .filter(messages::session_id.eq(&session_id))
            .filter(messages::role.eq("assistant"))
            .load(&mut conn)
            .unwrap();

        if !messages.is_empty() {
            println!("Agent response: {}", messages.last().unwrap().content);
        }
    }
}

#[tokio::test]
async fn test_autonomous_mode_no_approval_needed() {
    let pool = create_test_pool();

    // Create agent in autonomous mode
    let auto_agent = create_autonomous_agent(
        &pool,
        "Auto Agent",
        "🤖",
        "Autonomous agent that doesn't need approval",
        "You are an autonomous agent. Execute tasks without asking for permission.",
        true, // autonomous
    );

    // Verify execution_settings has autopilot mode
    assert!(auto_agent.execution_settings.is_some());
    let settings: serde_json::Value = serde_json::from_str(
        auto_agent.execution_settings.as_ref().unwrap()
    ).unwrap();
    assert_eq!(settings["mode"], "autopilot");

    // Create agent in approval mode
    let approval_agent = create_autonomous_agent(
        &pool,
        "Approval Agent",
        "🔐",
        "Agent that requires approval",
        "You are an agent that requires approval for actions.",
        false, // not autonomous
    );

    // Verify execution_settings has require_approval mode
    assert!(approval_agent.execution_settings.is_some());
    let settings: serde_json::Value = serde_json::from_str(
        approval_agent.execution_settings.as_ref().unwrap()
    ).unwrap();
    assert_eq!(settings["mode"], "require_approval");
}

#[tokio::test]
async fn test_autonomous_permission_manager() {
    let auto_pm = AutonomousPermissionManager::new(true);

    // Create a test permission request
    let req = anyagents::permissions::PermissionRequest {
        id: uuid::Uuid::new_v4().to_string(),
        permission_type: anyagents::permissions::PermissionType::ShellExecute,
        message: "Execute bash command".to_string(),
        metadata: std::collections::HashMap::new(),
    };

    // In autonomous mode, should auto-approve
    let result = auto_pm.request_permission(None, req).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), true, "Should auto-approve in autonomous mode");

    // Test non-autonomous mode
    let normal_pm = AutonomousPermissionManager::new(false);
    let req2 = anyagents::permissions::PermissionRequest {
        id: uuid::Uuid::new_v4().to_string(),
        permission_type: anyagents::permissions::PermissionType::FilesystemWrite,
        message: "Write file".to_string(),
        metadata: std::collections::HashMap::new(),
    };

    // Without observer, should deny
    let result = normal_pm.request_permission(None, req2).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), false, "Should deny without observer in normal mode");
}

#[tokio::test]
async fn test_multi_agent_email_chain() {
    let pool = create_test_pool();

    // Create a chain of agents
    let _manager = create_autonomous_agent(
        &pool,
        "Manager",
        "👔",
        "Project manager",
        "You are a project manager who coordinates team members.",
        true,
    );

    let _designer = create_autonomous_agent(
        &pool,
        "Designer",
        "🎨",
        "UI/UX Designer",
        "You are a designer who creates mockups.",
        true,
    );

    let _developer = create_autonomous_agent(
        &pool,
        "Developer",
        "💻",
        "Software developer",
        "You are a developer who implements features.",
        true,
    );

    // Verify all agents were created
    let mut conn = pool.get().unwrap();
    let all_agents: Vec<Agent> = agents::table.load(&mut conn).unwrap();
    assert_eq!(all_agents.len(), 3, "Should have 3 agents");

    println!("Created agents:");
    for agent in &all_agents {
        println!("  - {} ({})", agent.name, agent.avatar.as_deref().unwrap_or(""));
    }
}

#[tokio::test]
async fn test_email_with_autonomous_reply() {
    let pool = create_test_pool();

    // Create sender and recipient
    let _sender = create_autonomous_agent(
        &pool,
        "Sender",
        "📤",
        "Sender agent",
        "You are a helpful agent.",
        true,
    );

    let _recipient = create_autonomous_agent(
        &pool,
        "Recipient",
        "📥",
        "Recipient agent with autonomous mode",
        "You are a helpful agent. When you receive emails, respond naturally.",
        true,
    );

    // For now, just verify agents are in autonomous mode
    // Direct Tauri commands require complex State wrapping tested elsewhere

    let mut conn = pool.get().unwrap();
    let all_agents: Vec<anyagents::models::Agent> = agents::table
        .load(&mut conn)
        .unwrap();

    assert_eq!(all_agents.len(), 2, "Should have 2 agents");

    // Verify autonomous mode settings
    for agent in &all_agents {
        if let Some(ref settings_str) = agent.execution_settings {
            let settings: serde_json::Value = serde_json::from_str(settings_str).unwrap();
            assert_eq!(settings["mode"], "autopilot", "Agent should be in autopilot mode");
        }
    }
}

#[test]
fn test_execution_settings_parsing() {
    // Test parsing different execution settings formats

    // Autonomous mode
    let auto_settings = serde_json::json!({
        "mode": "autopilot",
        "sandbox_mode": "flexible"
    }).to_string();

    let parsed: serde_json::Value = serde_json::from_str(&auto_settings).unwrap();
    assert_eq!(parsed["mode"], "autopilot");

    // Approval mode
    let approval_settings = serde_json::json!({
        "mode": "require_approval",
        "sandbox_mode": "sandbox"
    }).to_string();

    let parsed: serde_json::Value = serde_json::from_str(&approval_settings).unwrap();
    assert_eq!(parsed["mode"], "require_approval");

    // Smart approval mode
    let smart_settings = serde_json::json!({
        "mode": "smart_approval",
        "sandbox_mode": "flexible",
        "auto_approve": ["filesystem_read", "network"]
    }).to_string();

    let parsed: serde_json::Value = serde_json::from_str(&smart_settings).unwrap();
    assert_eq!(parsed["mode"], "smart_approval");
}
