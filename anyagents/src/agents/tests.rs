/// Comprehensive test suite for agent system
use super::*;
use crate::database::create_test_pool;
use crate::models::{Agent, NewAgent};
use crate::permissions::PermissionManager;

/// Helper function to create a test agent
fn create_test_agent_db(pool: &DbPool, name: &str) -> Agent {
    use crate::schema::agents;
    use diesel::prelude::*;

    let ai_config_json = serde_json::json!({
        "provider": "openai",
        "model": "gpt-4o"
    })
    .to_string();

    let new_agent = NewAgent {
        id: uuid::Uuid::new_v4().to_string(),
        name: name.to_string(),
        description: Some(format!("Test agent: {}", name)),
        status: "active".to_string(),
        personality: Some("professional".to_string()),
        tone: Some("friendly".to_string()),
        expertise: Some("rust,testing".to_string()),
        ai_provider: "openai".to_string(),
        ai_model: "gpt-4o".to_string(),
        ai_temperature: 0.7,
        ai_config: ai_config_json,
        system_prompt: Some("You are a helpful assistant for testing.".to_string()),
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
        avatar: None,
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

#[cfg(test)]
mod agent_creation_tests {
    use super::*;

    #[test]
    fn test_agent_creation() {
        let pool = create_test_pool();
        let agent = create_test_agent_db(&pool, "TestAgent");

        assert_eq!(agent.name, "TestAgent");
        assert_eq!(agent.status, "active");
        assert_eq!(agent.ai_provider, "openai");
        assert_eq!(agent.ai_model, "gpt-4o");
        assert_eq!(agent.ai_temperature, 0.7);
    }

    #[test]
    fn test_agent_dto_conversion() {
        let pool = create_test_pool();
        let agent = create_test_agent_db(&pool, "DTOTestAgent");
        let dto = agent.clone().into_dto();

        assert_eq!(dto.id, agent.id);
        assert_eq!(dto.name, agent.name);
        assert_eq!(dto.ai_config.provider, "openai");
        assert_eq!(dto.ai_config.model, "gpt-4o");
        assert_eq!(
            dto.characteristics.personality,
            Some("professional".to_string())
        );
    }

    #[test]
    fn test_multiple_agents_creation() {
        let pool = create_test_pool();
        let agent1 = create_test_agent_db(&pool, "Agent1");
        let agent2 = create_test_agent_db(&pool, "Agent2");

        assert_ne!(agent1.id, agent2.id);
        assert_eq!(agent1.name, "Agent1");
        assert_eq!(agent2.name, "Agent2");
    }
}

#[cfg(test)]
mod agent_loop_tests {
    use super::*;

    #[tokio::test]
    async fn test_agent_loop_initialization() {
        let pool = create_test_pool();
        let agent = create_test_agent_db(&pool, "LoopTestAgent");

        let agent_loop = AgentLoop::new(&agent, pool.clone()).await;

        assert_eq!(agent_loop.agent_id, agent.id);
        assert_eq!(agent_loop.model, agent.ai_model);
        assert_eq!(agent_loop.history.len(), 0);
        assert_eq!(agent_loop.tools.len(), 5); // Filesystem, Search, Bash, Transcribe, SendEmail
    }

    #[tokio::test]
    async fn test_agent_loop_tools_registered() {
        let pool = create_test_pool();
        let agent = create_test_agent_db(&pool, "ToolsTestAgent");

        let agent_loop = AgentLoop::new(&agent, pool.clone()).await;

        let tool_names: Vec<String> = agent_loop
            .tools
            .iter()
            .map(|t| t.name().to_string())
            .collect();

        assert!(tool_names.contains(&"filesystem".to_string()));
        assert!(tool_names.contains(&"search_files".to_string()));
        assert!(tool_names.contains(&"bash".to_string()));
    }
}

#[cfg(test)]
mod message_history_tests {
    use super::*;
    use rig::completion::Message;

    #[test]
    fn test_message_history_management() {
        let mut history: Vec<Message> = vec![];

        // Add user message
        history.push(create_user_message("Hello".to_string()));

        // Add assistant message
        history.push(create_assistant_message("Hi there!".to_string()));

        assert_eq!(history.len(), 2);
        assert!(matches!(history[0], Message::User { .. }));
        assert!(matches!(history[1], Message::Assistant { .. }));
        // Verify content helper works - get_message_content uses Debug format
        assert!(get_message_content(&history[0]).contains("Hello"));
    }

    #[test]
    fn test_message_history_ordering() {
        let mut history: Vec<Message> = vec![];

        for i in 0..5 {
            if i % 2 == 0 {
                history.push(create_user_message(format!("Message {}", i)));
            } else {
                history.push(create_assistant_message(format!("Message {}", i)));
            }
        }

        assert_eq!(history.len(), 5);
        assert!(matches!(history[0], Message::User { .. }));
        assert!(matches!(history[1], Message::Assistant { .. }));
        // assert_eq!(get_message_content(&history[4]), "Message 4");
    }
}

#[cfg(test)]
mod tool_integration_tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_tool_schema_generation() {
        let workspace = PathBuf::from("/tmp/test_workspace");
        let tools: Vec<Box<dyn Tool>> = vec![
            Box::new(crate::tools::filesystem::FilesystemTool::new(workspace.clone())),
            Box::new(crate::tools::search::SearchTool),
            Box::new(crate::tools::bash::BashTool::new(workspace, "direct".to_string())),
        ];

        for tool in tools.iter() {
            let name = tool.name();
            let description = tool.description();
            let schema = tool.parameters_schema();

            assert!(!name.is_empty(), "Tool name should not be empty");
            assert!(
                !description.is_empty(),
                "Tool description should not be empty"
            );
            assert!(schema.is_object(), "Tool schema should be a JSON object");
        }
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_agent_creation_performance() {
        let pool = create_test_pool();
        let start = Instant::now();

        for i in 0..10 {
            let _ = create_test_agent_db(&pool, &format!("PerfAgent{}", i));
        }

        let duration = start.elapsed();
        println!("Created 10 agents in {:?}", duration);

        // Should create 10 agents in less than 1 second
        assert!(
            duration.as_secs() < 1,
            "Agent creation took too long: {:?}",
            duration
        );
    }

    #[tokio::test]
    async fn test_agent_loop_initialization_performance() {
        let pool = create_test_pool();
        let agent = create_test_agent_db(&pool, "PerfLoopAgent");

        let start = Instant::now();

        for _ in 0..100 {
            let _ = AgentLoop::new(&agent, pool.clone()).await;
        }

        let duration = start.elapsed();
        println!("Initialized 100 agent loops in {:?}", duration);

        // Should initialize 100 loops in less than 1 second
        assert!(
            duration.as_secs() < 1,
            "Agent loop initialization took too long: {:?}",
            duration
        );
    }
}

#[cfg(test)]
mod database_tests {
    use super::*;
    use crate::schema::agents;
    use diesel::prelude::*;

    #[test]
    fn test_agent_persistence() {
        let pool = create_test_pool();
        let agent = create_test_agent_db(&pool, "PersistenceTest");

        // Retrieve agent from database
        let mut conn = pool.get().expect("Failed to get connection");
        let retrieved: Agent = agents::table
            .filter(agents::id.eq(&agent.id))
            .first(&mut conn)
            .expect("Failed to retrieve agent");

        assert_eq!(retrieved.id, agent.id);
        assert_eq!(retrieved.name, agent.name);
        assert_eq!(retrieved.ai_model, agent.ai_model);
    }

    #[test]
    fn test_agent_update() {
        let pool = create_test_pool();
        let agent = create_test_agent_db(&pool, "UpdateTest");
        let mut conn = pool.get().expect("Failed to get connection");

        // Wait a moment to ensure timestamp difference
        std::thread::sleep(std::time::Duration::from_millis(10));
        let new_timestamp = chrono::Utc::now().timestamp();

        // Update agent name
        diesel::update(agents::table.filter(agents::id.eq(&agent.id)))
            .set((
                agents::name.eq("UpdatedName"),
                agents::updated_at.eq(new_timestamp),
            ))
            .execute(&mut conn)
            .expect("Failed to update agent");

        // Verify update
        let updated: Agent = agents::table
            .filter(agents::id.eq(&agent.id))
            .first(&mut conn)
            .expect("Failed to retrieve updated agent");

        assert_eq!(updated.name, "UpdatedName");
        assert!(updated.updated_at >= agent.updated_at);
    }

    #[test]
    fn test_agent_deletion() {
        let pool = create_test_pool();
        let agent = create_test_agent_db(&pool, "DeleteTest");
        let mut conn = pool.get().expect("Failed to get connection");

        // Delete agent
        diesel::delete(agents::table.filter(agents::id.eq(&agent.id)))
            .execute(&mut conn)
            .expect("Failed to delete agent");

        // Verify deletion
        let result: Result<Agent, _> = agents::table
            .filter(agents::id.eq(&agent.id))
            .first(&mut conn);

        assert!(result.is_err(), "Agent should be deleted");
    }
}
