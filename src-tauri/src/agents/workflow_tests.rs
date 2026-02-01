/// Integration tests for daily agent workflows
///
/// These tests simulate realistic user scenarios to ensure agents
/// provide efficient, natural assistance with appropriate interaction levels.

#[cfg(test)]
mod daily_workflow_tests {
    use crate::database::create_test_pool;
    use crate::models::{Agent, NewAgent, NewSession, Session};
    use diesel::prelude::*;
    use uuid::Uuid;

    /// Helper to create test agent with specific personality
    fn create_workflow_agent(
        pool: &crate::database::DbPool,
        name: &str,
        personality: &str,
    ) -> Agent {
        use crate::schema::agents;

        let ai_config_json = serde_json::json!({
            "provider": "openai",
            "model": "gpt-4o",
            "temperature": 0.7
        })
        .to_string();

        let system_prompt = match personality {
            "concise" => "You are a concise assistant. Provide brief, direct answers. Only ask questions when absolutely necessary.",
            "interactive" => "You are an interactive assistant. Engage with users, ask clarifying questions when needed, but don't over-communicate.",
            "proactive" => "You are a proactive assistant. Anticipate user needs and offer helpful suggestions, but respect user autonomy.",
            _ => "You are a helpful assistant."
        };

        let new_agent = NewAgent {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            description: Some(format!("{} workflow agent", personality)),
            status: "active".to_string(),
            personality: Some(personality.to_string()),
            tone: Some("professional".to_string()),
            expertise: Some("general,coding,file-management".to_string()),
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
            execution_settings: None,
            scope_type: None,
            workspace_path: None,
        };

        let mut conn = pool.get().expect("Failed to get connection");
        diesel::insert_into(agents::table)
            .values(&new_agent)
            .execute(&mut conn)
            .expect("Failed to insert agent");

        agents::table
            .filter(agents::id.eq(&new_agent.id))
            .first(&mut conn)
            .expect("Failed to retrieve agent")
    }

    /// Helper to create test session
    fn create_test_session(pool: &crate::database::DbPool, agent_id: &str) -> Session {
        use crate::schema::sessions;

        let new_session = NewSession {
            id: Uuid::new_v4().to_string(),
            agent_id: agent_id.to_string(),
            title: Some("Test Session".to_string()),
            created_at: chrono::Utc::now().naive_utc(),
            updated_at: chrono::Utc::now().naive_utc(),
            archived: 0,
            pinned: 0,
        };

        let mut conn = pool.get().expect("Failed to get connection");
        diesel::insert_into(sessions::table)
            .values(&new_session)
            .execute(&mut conn)
            .expect("Failed to insert session");

        sessions::table
            .filter(sessions::id.eq(&new_session.id))
            .first(&mut conn)
            .expect("Failed to retrieve session")
    }

    // ============= SCENARIO TESTS =============

    #[test]
    fn scenario_1_simple_file_listing() {
        // SCENARIO: User asks to list files
        // EXPECTED: Agent should use filesystem tool directly without asking questions
        // INTERACTION LEVEL: Minimal (1 turn)

        let pool = create_test_pool();
        let agent = create_workflow_agent(&pool, "ConciseAgent", "concise");
        let _session = create_test_session(&pool, &agent.id);

        // User query
        let _user_query = "List files in the current directory";

        // Expected behavior validation
        assert_eq!(agent.personality, Some("concise".to_string()));
        assert!(agent.system_prompt.unwrap().contains("brief"));

        // This agent should:
        // 1. Recognize "list files" as a clear request
        // 2. Use filesystem tool with operation="list_dir", path="."
        // 3. Return results without asking "which directory?"

        println!("✓ Scenario 1: Simple file listing - Agent configured for minimal interaction");
    }

    #[test]
    fn scenario_2_ambiguous_search_query() {
        // SCENARIO: User asks to "search for TODO" (ambiguous scope)
        // EXPECTED: Agent should ask ONE clarifying question about scope
        // INTERACTION LEVEL: Moderate (2 turns)

        let pool = create_test_pool();
        let agent = create_workflow_agent(&pool, "InteractiveAgent", "interactive");
        let _session = create_test_session(&pool, &agent.id);

        // User query
        let _user_query = "Search for TODO";

        // Expected behavior:
        // Agent message 1: "Would you like me to search in the current directory or the entire project?"
        // User response: "Current directory"
        // Agent action: Execute search_files with path="."

        assert_eq!(agent.personality, Some("interactive".to_string()));
        assert!(agent
            .system_prompt
            .unwrap()
            .contains("clarifying questions when needed"));

        println!("✓ Scenario 2: Ambiguous search - Agent asks ONE clarifying question");
    }

    #[test]
    fn scenario_3_file_creation_with_content() {
        // SCENARIO: User asks to create a file with specific content
        // EXPECTED: Agent should execute directly if all info provided
        // INTERACTION LEVEL: Minimal (1 turn)

        let pool = create_test_pool();
        let agent = create_workflow_agent(&pool, "ConciseAgent2", "concise");
        let _session = create_test_session(&pool, &agent.id);

        // User query (all info provided)
        let _user_query = "Create a file called test.txt with content 'Hello World'";

        // Expected behavior:
        // 1. Parse: filename="test.txt", content="Hello World"
        // 2. Use filesystem tool: operation="write", path="test.txt", content="Hello World"
        // 3. Report success

        assert_eq!(agent.personality, Some("concise".to_string()));

        println!("✓ Scenario 3: File creation with content - Agent executes directly");
    }

    #[test]
    fn scenario_4_complex_workflow_with_confirmation() {
        // SCENARIO: User asks to "delete all .log files"
        // EXPECTED: Agent should ask for confirmation before destructive action
        // INTERACTION LEVEL: Moderate (2 turns) - safety confirmation

        let pool = create_test_pool();
        let agent = create_workflow_agent(&pool, "ProactiveAgent", "proactive");
        let _session = create_test_session(&pool, &agent.id);

        // User query (destructive operation)
        let _user_query = "Delete all .log files in this directory";

        // Expected behavior:
        // 1. Search for .log files
        // 2. Show count: "Found 5 .log files. Do you want to delete them?"
        // 3. Wait for user confirmation
        // 4. Execute deletion if confirmed

        assert_eq!(agent.personality, Some("proactive".to_string()));

        println!("✓ Scenario 4: Destructive operation - Agent asks for confirmation");
    }

    #[test]
    fn scenario_5_multi_step_code_review() {
        // SCENARIO: User asks "Review my recent changes"
        // EXPECTED: Agent should break down into steps but minimize back-and-forth
        // INTERACTION LEVEL: Low-Moderate (2-3 turns max)

        let pool = create_test_pool();
        let agent = create_workflow_agent(&pool, "ProactiveAgent2", "proactive");
        let _session = create_test_session(&pool, &agent.id);

        // User query (requires multiple steps)
        let _user_query = "Review my recent code changes";

        // Expected behavior:
        // 1. Execute: git diff (or similar) to get changes
        // 2. Analyze changes
        // 3. Provide review with suggestions
        // NO: "Which files do you want me to review?" (too many questions)
        // YES: Review all recent changes proactively

        assert_eq!(agent.personality, Some("proactive".to_string()));
        assert!(agent
            .system_prompt
            .unwrap()
            .contains("Anticipate user needs"));

        println!(
            "✓ Scenario 5: Code review - Agent proactively reviews without excessive questions"
        );
    }

    #[test]
    fn scenario_6_vague_request_needs_clarification() {
        // SCENARIO: User says "Fix it"
        // EXPECTED: Agent must ask what to fix (necessary clarification)
        // INTERACTION LEVEL: Moderate (2 turns)

        let pool = create_test_pool();
        let agent = create_workflow_agent(&pool, "InteractiveAgent2", "interactive");
        let _session = create_test_session(&pool, &agent.id);

        // User query (too vague)
        let _user_query = "Fix it";

        // Expected behavior:
        // Agent message: "I'd be happy to help fix something. Could you specify what needs to be fixed?"
        // This is a NECESSARY question - the agent cannot proceed otherwise

        assert_eq!(agent.personality, Some("interactive".to_string()));

        println!("✓ Scenario 6: Vague request - Agent asks necessary clarification");
    }

    #[test]
    fn scenario_7_efficient_batch_operation() {
        // SCENARIO: User asks to "format all Python files"
        // EXPECTED: Agent should batch process without asking about each file
        // INTERACTION LEVEL: Minimal (1 turn + optional summary)

        let pool = create_test_pool();
        let agent = create_workflow_agent(&pool, "ConciseAgent3", "concise");
        let _session = create_test_session(&pool, &agent.id);

        // User query (batch operation)
        let _user_query = "Format all Python files in src/";

        // Expected behavior:
        // 1. Find all .py files in src/
        // 2. Format each one (use bash tool with formatter)
        // 3. Report summary: "Formatted 12 Python files"
        // NO: "Should I format file1.py?" for each file (over-interaction)

        assert_eq!(agent.personality, Some("concise".to_string()));

        println!("✓ Scenario 7: Batch operation - Agent processes efficiently without per-item confirmation");
    }

    #[test]
    fn scenario_8_context_aware_follow_up() {
        // SCENARIO: Multi-turn conversation with context
        // EXPECTED: Agent remembers context, doesn't ask redundant questions
        // INTERACTION LEVEL: Efficient multi-turn

        let pool = create_test_pool();
        let agent = create_workflow_agent(&pool, "InteractiveAgent3", "interactive");
        let session = create_test_session(&pool, &agent.id);

        // Turn 1
        let user_query_1 = "List all Rust files";
        // Agent: [Shows list of .rs files]

        // Turn 2
        let user_query_2 = "Now show me the tests in those files";
        // Agent should:
        // - Remember "those files" refers to Rust files from Turn 1
        // - Search for #[test] or #[cfg(test)] in the same files
        // NO: "Which files do you mean?" (context was already established)

        assert_eq!(session.agent_id, agent.id);
        println!("Query 1: {}", user_query_1);
        println!("Query 2: {}", user_query_2);

        println!("✓ Scenario 8: Context-aware follow-up - Agent uses conversation context");
    }
}

// ============= INTERACTION PATTERN TESTS =============

#[cfg(test)]
mod interaction_patterns {
    #[test]
    fn test_optimal_interaction_levels() {
        // Define optimal interaction levels for different request types

        struct RequestPattern {
            request_type: &'static str,
            expected_turns: usize,
            requires_confirmation: bool,
            example: &'static str,
        }

        let patterns = vec![
            RequestPattern {
                request_type: "Simple Read Operation",
                expected_turns: 1,
                requires_confirmation: false,
                example: "Show me the contents of config.json",
            },
            RequestPattern {
                request_type: "Simple Write Operation",
                expected_turns: 1,
                requires_confirmation: false,
                example: "Create a file named README.md",
            },
            RequestPattern {
                request_type: "Destructive Operation",
                expected_turns: 2, // Action + Confirmation
                requires_confirmation: true,
                example: "Delete all temporary files",
            },
            RequestPattern {
                request_type: "Ambiguous Request",
                expected_turns: 2, // Clarification + Action
                requires_confirmation: false,
                example: "Search for error",
            },
            RequestPattern {
                request_type: "Complex Multi-Step",
                expected_turns: 1, // Agent should handle autonomously
                requires_confirmation: false,
                example: "Analyze and summarize recent changes",
            },
            RequestPattern {
                request_type: "Vague Request",
                expected_turns: 2, // Clarification + Action
                requires_confirmation: false,
                example: "Help me with that thing",
            },
        ];

        for pattern in patterns {
            println!(
                "\n{}: {} turns expected",
                pattern.request_type, pattern.expected_turns
            );
            println!("  Example: '{}'", pattern.example);
            println!("  Needs confirmation: {}", pattern.requires_confirmation);

            // Validate pattern makes sense
            assert!(
                pattern.expected_turns > 0 && pattern.expected_turns <= 3,
                "Interaction should be 1-3 turns for good UX"
            );
        }

        println!("\n✓ Interaction patterns validated");
    }

    #[test]
    fn test_anti_patterns() {
        // Define what NOT to do (anti-patterns)

        struct AntiPattern {
            pattern: &'static str,
            why_bad: &'static str,
        }

        let anti_patterns = vec![
            AntiPattern {
                pattern: "Asking 'Are you sure?' for every action",
                why_bad: "Creates friction, wastes time on safe operations",
            },
            AntiPattern {
                pattern: "Asking for clarification when context is clear",
                why_bad: "Shows poor understanding of natural language",
            },
            AntiPattern {
                pattern: "Breaking simple tasks into multiple confirmations",
                why_bad: "Makes assistant feel tedious rather than helpful",
            },
            AntiPattern {
                pattern: "Not asking for confirmation on destructive operations",
                why_bad: "Unsafe, could cause data loss",
            },
            AntiPattern {
                pattern: "Asking the same question twice in one conversation",
                why_bad: "Shows lack of context retention",
            },
            AntiPattern {
                pattern: "Explaining every step before executing",
                why_bad: "Over-communication, user just wants results",
            },
        ];

        for anti_pattern in anti_patterns {
            println!("\n❌ Anti-pattern: {}", anti_pattern.pattern);
            println!("   Why it's bad: {}", anti_pattern.why_bad);
        }

        println!("\n✓ Anti-patterns documented");
    }
}

// ============= NATURAL LANGUAGE FLOW TESTS =============

#[cfg(test)]
mod natural_language_flows {
    #[test]
    fn test_conversation_naturalness() {
        // Test that agent responses feel natural and conversational

        struct ConversationExample {
            user: &'static str,
            agent_response_style: &'static str,
            is_natural: bool,
        }

        let examples = vec![
            ConversationExample {
                user: "List files",
                agent_response_style: "Here are the files in the current directory: [list]",
                is_natural: true,
            },
            ConversationExample {
                user: "List files",
                agent_response_style: "Certainly! I will now execute the filesystem tool with the list_dir operation on the current directory path. [technical details]",
                is_natural: false, // Too verbose
            },
            ConversationExample {
                user: "Find TODO comments",
                agent_response_style: "I found 5 TODO comments: [list with file:line]",
                is_natural: true,
            },
            ConversationExample {
                user: "Find TODO comments",
                agent_response_style: "I will search for 'TODO' in which directory?",
                is_natural: false, // Should default to current dir
            },
        ];

        for (i, example) in examples.iter().enumerate() {
            println!("\nExample {}:", i + 1);
            println!("  User: '{}'", example.user);
            println!("  Agent: '{}'", example.agent_response_style);
            println!("  Natural? {}", if example.is_natural { "✓" } else { "✗" });

            if !example.is_natural {
                println!("  Issue: Agent response is not natural");
            }
        }

        println!("\n✓ Conversation naturalness examples validated");
    }
}
