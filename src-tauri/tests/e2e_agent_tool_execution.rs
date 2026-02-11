//! End-to-end tests for actual agent tool execution
//!
//! These tests invoke real agents that execute real tools (file operations, bash, search)
//! Unlike the workflow tests which simulate via email, these tests verify actual tool calls.

use anyagents::agents::coordinator::Coordinator;
use anyagents::database::create_test_pool;
use anyagents::events::AgentObserver;
use anyagents::models::{Agent, NewAgent};
use anyagents::permissions::AutonomousPermissionManager;
use anyagents::schema::agents;
use diesel::prelude::*;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Test observer that captures events for verification
struct TestObserver {
    events: Arc<Mutex<Vec<(String, Value)>>>, // Store (event_type, payload) pairs
}

impl TestObserver {
    fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    async fn get_tool_calls(&self) -> Vec<(String, Value)> {
        let events = self.events.lock().await;
        events
            .iter()
            .filter(|(event_type, payload)| {
                event_type == "step_started" &&
                payload.get("step").and_then(|s| s.get("tool_name")).is_some()
            })
            .map(|(_, payload)| {
                let tool_name = payload["step"]["tool_name"].as_str().unwrap_or("unknown").to_string();
                let tool_args = payload["step"]["tool_args"].clone();
                (tool_name, tool_args)
            })
            .collect()
    }
}

impl AgentObserver for TestObserver {
    fn emit(&self, event: &str, payload: Value) -> Result<(), String> {
        let events = self.events.clone();
        let event_type = event.to_string();
        tokio::spawn(async move {
            events.lock().await.push((event_type, payload));
        });
        Ok(())
    }
}

/// Helper to create an agent configured for file operations
fn create_file_agent(pool: &anyagents::database::DbPool, workspace: &str) -> Agent {
    let ai_config_json = serde_json::json!({
        "provider": "openai",
        "model": "gpt-4o",
        "temperature": 0.3
    })
    .to_string();

    // Autonomous execution settings (auto-approve all permissions)
    let execution_settings = serde_json::json!({
        "mode": "autopilot",
        "sandbox_mode": "direct"
    })
    .to_string();

    let new_agent = NewAgent {
        id: uuid::Uuid::new_v4().to_string(),
        name: "File Operations Agent".to_string(),
        description: Some("Agent that performs file operations".to_string()),
        status: "active".to_string(),
        personality: Some("professional".to_string()),
        tone: Some("concise".to_string()),
        expertise: Some("file-operations,coding".to_string()),
        ai_provider: "openai".to_string(),
        ai_model: "gpt-4o".to_string(),
        ai_temperature: 0.3,
        ai_config: ai_config_json,
        system_prompt: Some(
            "You are a helpful file operations agent. When asked to perform file operations, use the filesystem tool directly without asking for confirmation. Be concise in responses."
                .to_string(),
        ),
        permissions: None,
        working_directories: Some(workspace.to_string()),
        skills: None,
        mcp_servers: None,
        messaging_connections: None,
        knowledge_bases: None,
        api_keys: None,
        created_at: chrono::Utc::now().timestamp(),
        updated_at: chrono::Utc::now().timestamp(),
        platform_configs: None,
        execution_settings: Some(execution_settings),
        scope_type: None,
        workspace_path: Some(workspace.to_string()),
        avatar: Some("📁".to_string()),
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

/// Test 1: Agent creates a file using filesystem tool
#[tokio::test]
async fn test_agent_creates_file_with_tool() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    if std::env::var("OPENAI_API_KEY").is_err() {
        println!("SKIPPED: No OPENAI_API_KEY found");
        return Ok(());
    }

    println!("\n🔧 Test: Agent creates file using filesystem tool");

    // Setup temp workspace
    let temp_dir = std::env::temp_dir().join(format!("anycowork_test_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir)?;
    let workspace = temp_dir.to_string_lossy().to_string();

    println!("✓ Workspace: {}", workspace);

    // Create agent and database
    let pool = create_test_pool();
    let agent = create_file_agent(&pool, &workspace);

    println!("✓ Created agent: {}", agent.name);

    // Create observer to capture events
    let observer = Arc::new(TestObserver::new());
    let session_id = uuid::Uuid::new_v4().to_string();

    // Create autonomous permission manager (auto-approves everything)
    let autonomous_pm = Arc::new(AutonomousPermissionManager::new(true));

    // Create coordinator
    let coordinator = Coordinator::new_with_autonomous(
        session_id.clone(),
        agent,
        observer.clone(),
        pool.clone(),
        autonomous_pm,
        Arc::new(dashmap::DashMap::new()),
        "direct".to_string(), // Direct execution mode
        None,
    );

    println!("✓ Coordinator created");

    // Ask agent to create a file
    let prompt = "Create a file called test_file.txt with content 'Hello from agent tool test!'";
    println!("\n📝 Prompt: {}", prompt);

    // Run agent (this will make actual LLM API call and execute tools)
    coordinator.run(prompt.to_string()).await;

    println!("✓ Agent execution completed");

    // Wait a moment for events to be captured
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    // Get captured tool calls
    let tool_calls = observer.get_tool_calls().await;
    println!("\n📊 Captured {} tool calls", tool_calls.len());

    // Verify filesystem tool was called
    let found_filesystem = tool_calls.iter().any(|(tool_name, _)| tool_name == "filesystem");

    if found_filesystem {
        println!("✓ Found filesystem tool call");
        for (tool_name, args) in &tool_calls {
            if tool_name == "filesystem" {
                println!("   Args: {:?}", args);
            }
        }
    }

    assert!(
        found_filesystem,
        "Agent should have called filesystem tool"
    );

    // Verify file was actually created
    let file_path = temp_dir.join("test_file.txt");
    assert!(
        file_path.exists(),
        "File should exist at {:?}",
        file_path
    );

    let content = std::fs::read_to_string(&file_path)?;
    assert!(
        content.contains("Hello from agent tool test"),
        "File should contain expected content"
    );

    println!("✓ File was created with correct content");

    // Cleanup
    std::fs::remove_dir_all(&temp_dir)?;

    println!("\n✅ Test PASSED: Agent successfully created file using filesystem tool");
    Ok(())
}

/// Test 2: Agent reads a file using filesystem tool
#[tokio::test]
async fn test_agent_reads_file_with_tool() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    if std::env::var("OPENAI_API_KEY").is_err() {
        println!("SKIPPED: No OPENAI_API_KEY found");
        return Ok(());
    }

    println!("\n🔧 Test: Agent reads file using filesystem tool");

    // Setup temp workspace with existing file
    let temp_dir = std::env::temp_dir().join(format!("anycowork_test_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir)?;
    let workspace = temp_dir.to_string_lossy().to_string();

    // Create a test file
    let test_file = temp_dir.join("existing_file.txt");
    std::fs::write(&test_file, "This is test content for reading.")?;

    println!("✓ Created test file: {:?}", test_file);

    // Create agent
    let pool = create_test_pool();
    let agent = create_file_agent(&pool, &workspace);

    let observer = Arc::new(TestObserver::new());
    let session_id = uuid::Uuid::new_v4().to_string();
    let autonomous_pm = Arc::new(AutonomousPermissionManager::new(true));

    let coordinator = Coordinator::new_with_autonomous(
        session_id,
        agent,
        observer.clone(),
        pool,
        autonomous_pm,
        Arc::new(dashmap::DashMap::new()),
        "direct".to_string(),
        None,
    );

    // Ask agent to read the file
    let prompt = "Read the file existing_file.txt and tell me what it contains";
    println!("\n📝 Prompt: {}", prompt);

    coordinator.run(prompt.to_string()).await;

    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    let tool_calls = observer.get_tool_calls().await;
    println!("\n📊 Captured {} tool calls", tool_calls.len());

    // Verify read_file operation was called
    let mut found_read = false;
    for (tool_name, args) in &tool_calls {
        if tool_name == "filesystem" {
            if let Some(op) = args.get("operation") {
                if op.as_str() == Some("read_file") {
                    found_read = true;
                    println!("✓ Found read_file operation");
                    println!("   Args: {:?}", args);
                }
            }
        }
    }

    assert!(found_read, "Agent should have called read_file");

    // Cleanup
    std::fs::remove_dir_all(&temp_dir)?;

    println!("\n✅ Test PASSED: Agent successfully read file using filesystem tool");
    Ok(())
}

/// Test 3: Agent searches files using search_files tool
#[tokio::test]
async fn test_agent_searches_files_with_tool() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    if std::env::var("OPENAI_API_KEY").is_err() {
        println!("SKIPPED: No OPENAI_API_KEY found");
        return Ok(());
    }

    println!("\n🔧 Test: Agent searches files using search_files tool");

    // Setup workspace with multiple files
    let temp_dir = std::env::temp_dir().join(format!("anycowork_test_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir)?;
    let workspace = temp_dir.to_string_lossy().to_string();

    // Create files with searchable content
    std::fs::write(temp_dir.join("file1.txt"), "This file contains TODO: Fix bug")?;
    std::fs::write(temp_dir.join("file2.txt"), "Normal content here")?;
    std::fs::write(temp_dir.join("file3.txt"), "Another TODO: Add feature")?;

    println!("✓ Created test files with TODO markers");

    // Create agent
    let pool = create_test_pool();
    let agent = create_file_agent(&pool, &workspace);

    let observer = Arc::new(TestObserver::new());
    let session_id = uuid::Uuid::new_v4().to_string();
    let autonomous_pm = Arc::new(AutonomousPermissionManager::new(true));

    let coordinator = Coordinator::new_with_autonomous(
        session_id,
        agent,
        observer.clone(),
        pool,
        autonomous_pm,
        Arc::new(dashmap::DashMap::new()),
        "direct".to_string(),
        None,
    );

    // Ask agent to search for TODO
    let prompt = "Search for all files containing the word TODO";
    println!("\n📝 Prompt: {}", prompt);

    coordinator.run(prompt.to_string()).await;

    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    let tool_calls = observer.get_tool_calls().await;
    println!("\n📊 Captured {} tool calls", tool_calls.len());

    // Verify search_files was called
    let found_search = tool_calls.iter().any(|(tool_name, _)| tool_name == "search_files");

    if found_search {
        println!("✓ Found search_files tool call");
        for (tool_name, args) in &tool_calls {
            if tool_name == "search_files" {
                println!("   Args: {:?}", args);
            }
        }
    }

    assert!(found_search, "Agent should have called search_files");

    // Cleanup
    std::fs::remove_dir_all(&temp_dir)?;

    println!("\n✅ Test PASSED: Agent successfully searched files using search_files tool");
    Ok(())
}

/// Test 4: Agent uses bash tool to execute command
#[tokio::test]
async fn test_agent_executes_bash_command() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    if std::env::var("OPENAI_API_KEY").is_err() {
        println!("SKIPPED: No OPENAI_API_KEY found");
        return Ok(());
    }

    println!("\n🔧 Test: Agent executes bash command");

    let temp_dir = std::env::temp_dir().join(format!("anycowork_test_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir)?;
    let workspace = temp_dir.to_string_lossy().to_string();

    let pool = create_test_pool();
    let agent = create_file_agent(&pool, &workspace);

    let observer = Arc::new(TestObserver::new());
    let session_id = uuid::Uuid::new_v4().to_string();
    let autonomous_pm = Arc::new(AutonomousPermissionManager::new(true));

    let coordinator = Coordinator::new_with_autonomous(
        session_id,
        agent,
        observer.clone(),
        pool,
        autonomous_pm,
        Arc::new(dashmap::DashMap::new()),
        "direct".to_string(),
        None,
    );

    // Ask agent to run a simple command
    #[cfg(target_os = "windows")]
    let prompt = "Run the command 'echo Hello from bash' and show me the output";
    #[cfg(not(target_os = "windows"))]
    let prompt = "Run the command 'echo Hello from bash' and show me the output";

    println!("\n📝 Prompt: {}", prompt);

    coordinator.run(prompt.to_string()).await;

    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    let tool_calls = observer.get_tool_calls().await;
    println!("\n📊 Captured {} tool calls", tool_calls.len());

    // Verify bash tool was called
    let found_bash = tool_calls.iter().any(|(tool_name, _)| tool_name == "bash");

    if found_bash {
        println!("✓ Found bash tool call");
        for (tool_name, args) in &tool_calls {
            if tool_name == "bash" {
                println!("   Args: {:?}", args);
            }
        }
    }

    assert!(found_bash, "Agent should have called bash tool");

    // Cleanup
    std::fs::remove_dir_all(&temp_dir)?;

    println!("\n✅ Test PASSED: Agent successfully executed bash command");
    Ok(())
}

/// Test 5: Multi-step workflow - Create file, then read it back
#[tokio::test]
async fn test_agent_multi_step_file_workflow() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    if std::env::var("OPENAI_API_KEY").is_err() {
        println!("SKIPPED: No OPENAI_API_KEY found");
        return Ok(());
    }

    println!("\n🔧 Test: Agent performs multi-step file workflow");

    let temp_dir = std::env::temp_dir().join(format!("anycowork_test_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir)?;
    let workspace = temp_dir.to_string_lossy().to_string();

    let pool = create_test_pool();
    let agent = create_file_agent(&pool, &workspace);

    let observer = Arc::new(TestObserver::new());
    let session_id = uuid::Uuid::new_v4().to_string();
    let autonomous_pm = Arc::new(AutonomousPermissionManager::new(true));

    let coordinator = Coordinator::new_with_autonomous(
        session_id,
        agent,
        observer.clone(),
        pool,
        autonomous_pm,
        Arc::new(dashmap::DashMap::new()),
        "direct".to_string(),
        None,
    );

    // Complex prompt requiring multiple tool calls
    let prompt = "Create a file called data.json with content '{\"name\": \"test\", \"value\": 42}' and then read it back to verify it was created correctly";
    println!("\n📝 Prompt: {}", prompt);

    coordinator.run(prompt.to_string()).await;

    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    let tool_calls = observer.get_tool_calls().await;
    println!("\n📊 Captured {} tool calls", tool_calls.len());

    // Verify both write and read operations
    let mut found_write = false;
    let mut found_read = false;

    for (tool_name, args) in &tool_calls {
        if tool_name == "filesystem" {
            if let Some(op) = args.get("operation") {
                if op.as_str() == Some("write_file") {
                    found_write = true;
                    println!("✓ Found write_file operation");
                    println!("   Args: {:?}", args);
                } else if op.as_str() == Some("read_file") {
                    found_read = true;
                    println!("✓ Found read_file operation");
                    println!("   Args: {:?}", args);
                }
            }
        }
    }

    assert!(found_write, "Agent should have written file");
    assert!(found_read, "Agent should have read file back");

    // Verify file exists and has correct content
    let file_path = temp_dir.join("data.json");
    assert!(file_path.exists(), "File should exist");

    let content = std::fs::read_to_string(&file_path)?;
    assert!(content.contains("test"), "File should contain expected content");

    println!("✓ File was created and read successfully");

    // Cleanup
    std::fs::remove_dir_all(&temp_dir)?;

    println!("\n✅ Test PASSED: Agent completed multi-step file workflow");
    Ok(())
}
