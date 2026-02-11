#[cfg(test)]
mod tests {
    use crate::agents::AgentLoop;
    use crate::database::{create_test_pool, DbPool};
    use crate::models::{Agent, NewAgent, NewAgentSkill, NewAgentSkillAssignment};
    use crate::schema::{agents, agent_skills, agent_skill_assignments};
    use diesel::prelude::*;
    use uuid::Uuid;

    fn create_test_agent(pool: &DbPool, name: &str) -> Agent {
        let mut conn = pool.get().unwrap();
        let id = Uuid::new_v4().to_string();
        
        let new_agent = NewAgent {
            id: id.clone(),
            name: name.to_string(),
            description: None,
            status: "active".to_string(),
            personality: None,
            tone: None,
            expertise: None,
            ai_provider: "openai".to_string(),
            ai_model: "gpt-4".to_string(),
            ai_temperature: 0.7,
            ai_config: "{}".to_string(),
            system_prompt: None,
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

        diesel::insert_into(agents::table)
            .values(&new_agent)
            .execute(&mut conn)
            .unwrap();

        agents::table
            .filter(agents::id.eq(id))
            .first(&mut conn)
            .unwrap()
    }

    fn create_test_skill(pool: &DbPool, name: &str) -> String {
        let mut conn = pool.get().unwrap();
        let id = Uuid::new_v4().to_string();

        let new_skill = NewAgentSkill {
            id: id.clone(),
            name: name.to_string(),
            display_title: name.to_string(),
            description: "Test skill".to_string(),
            skill_content: "# Test Skill".to_string(),
            additional_files_json: None,
            enabled: 1,
            version: 1,
            created_at: chrono::Utc::now().naive_utc(),
            updated_at: chrono::Utc::now().naive_utc(),
            source_path: None,
            category: None,
            requires_sandbox: 0,
            sandbox_config: None,
            execution_mode: "direct".to_string(),
        };

        diesel::insert_into(agent_skills::table)
            .values(&new_skill)
            .execute(&mut conn)
            .unwrap();

        id
    }

    fn assign_skill(pool: &DbPool, agent_id: &str, skill_id: &str) {
        let mut conn = pool.get().unwrap();
        let assignment = NewAgentSkillAssignment {
            agent_id: agent_id.to_string(),
            skill_id: skill_id.to_string(),
            created_at: chrono::Utc::now().naive_utc(),
        };

        diesel::insert_into(agent_skill_assignments::table)
            .values(&assignment)
            .execute(&mut conn)
            .unwrap();
    }

    #[tokio::test]
    async fn test_agent_loop_loads_assigned_skills() {
        let pool = create_test_pool();
        
        // 1. Create Agent and Skill
        let agent = create_test_agent(&pool, "SkillUserAgent");
        let skill_id = create_test_skill(&pool, "test-tool-skill");

        // 2. Assign Skill
        assign_skill(&pool, &agent.id, &skill_id);

        // 3. Init AgentLoop
        let agent_loop = AgentLoop::new(&agent, pool.clone()).await;

        // 4. Verify Tool Logic
        let tool_names: Vec<String> = agent_loop.tools.iter().map(|t| t.name().to_string()).collect();
        println!("Loaded tools: {:?}", tool_names);

        assert!(tool_names.contains(&"test-tool-skill".to_string()), "AgentLoop should load assigned skill as a tool");
    }
}
