use crate::database::DbPool;
use crate::events::AgentEvent;
use crate::llm::LlmClient;
use crate::models::Agent as DbAgent;
use diesel::prelude::*;
use rig::completion::Message;
use tauri::{Emitter, Runtime};
use uuid::Uuid;

/// Simple chat agent for conversational responses without tools
pub struct SimpleChatAgent {
    pub model: String,
    pub provider: String,
    pub system_prompt: Option<String>,
}

impl SimpleChatAgent {
    pub fn new(agent_db: &DbAgent) -> Self {
        Self {
            model: agent_db.ai_model.clone(),
            provider: agent_db.ai_provider.clone(),
            system_prompt: agent_db.system_prompt.clone(),
        }
    }

    pub async fn chat<R: Runtime>(
        &self,
        message: &str,
        session_id: &str,
        window: &tauri::WebviewWindow<R>,
        db_pool: &DbPool,
    ) -> Result<String, String> {
        // Load conversation history from DB
        let history = self.load_history(session_id, db_pool);

        let preamble = self.system_prompt.clone().unwrap_or_else(|| {
            "You are a helpful AI assistant. Respond naturally and conversationally to the user's questions. Be concise but thorough.".to_string()
        });

        // Create callback for streaming tokens
        let window_clone = window.clone();
        let session_id_owned = session_id.to_string();
        let on_token = move |token: String| {
            let _ = Emitter::emit(
                &window_clone,
                &format!("session:{}", session_id_owned),
                AgentEvent::Token {
                    content: token,
                },
            );
        };

        let client = LlmClient::new(&self.provider, &self.model).with_preamble(&preamble);
        let result = client.stream_chat(message, history, on_token).await;

        match result {
            Ok(full_response) => {
                // Save messages to DB
                self.save_message(db_pool, "user", message, session_id);
                self.save_message(db_pool, "assistant", &full_response, session_id);
                Ok(full_response)
            }
            Err(e) => Err(e),
        }
    }

    fn load_history(&self, session_id: &str, db_pool: &DbPool) -> Vec<Message> {
        use crate::schema::messages;

        let mut history = Vec::new();

        if let Ok(mut conn) = db_pool.get() {
            let db_messages: Result<Vec<crate::models::Message>, _> = messages::table
                .filter(messages::session_id.eq(session_id))
                .order(messages::created_at.asc())
                .limit(20) // Limit history for context
                .load(&mut conn);

            if let Ok(msgs) = db_messages {
                for msg in msgs {
                    let rig_msg = match msg.role.as_str() {
                        "user" => Message::user(&msg.content),
                        "assistant" | "model" => Message::assistant(&msg.content),
                        _ => continue,
                    };
                    history.push(rig_msg);
                }
            }
        }

        history
    }

    fn save_message(&self, db_pool: &DbPool, role: &str, content: &str, session_id: &str) {
        use crate::schema::messages;

        if let Ok(mut conn) = db_pool.get() {
            let msg = crate::models::NewMessage {
                id: Uuid::new_v4().to_string(),
                role: role.to_string(),
                content: content.to_string(),
                session_id: session_id.to_string(),
                metadata_json: None,
                tokens: None,
            };
            let _ = diesel::insert_into(messages::table)
                .values(&msg)
                .execute(&mut conn);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::create_test_pool;
    use crate::models::NewAgent;
    use crate::schema::agents;

    fn create_test_db_agent(pool: &DbPool) -> DbAgent {
        let mut conn = pool.get().unwrap();
        let id = uuid::Uuid::new_v4().to_string();

        let new_agent = NewAgent {
            id: id.clone(),
            name: "TestChatAgent".to_string(),
            description: Some("Test agent for chat".to_string()),
            status: "active".to_string(),
            personality: Some("friendly".to_string()),
            tone: Some("casual".to_string()),
            expertise: Some("general".to_string()),
            ai_provider: "openai".to_string(),
            ai_model: "gpt-4".to_string(),
            ai_temperature: 0.7,
            ai_config: "{}".to_string(),
            system_prompt: Some("You are a helpful test assistant.".to_string()),
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

        diesel::insert_into(agents::table)
            .values(&new_agent)
            .execute(&mut conn)
            .unwrap();

        agents::table
            .filter(agents::id.eq(id))
            .first(&mut conn)
            .unwrap()
    }

    #[test]
    fn test_simple_chat_agent_creation() {
        let pool = create_test_pool();
        let db_agent = create_test_db_agent(&pool);

        let chat_agent = SimpleChatAgent::new(&db_agent);

        assert_eq!(chat_agent.model, "gpt-4");
        assert_eq!(chat_agent.provider, "openai");
        assert_eq!(
            chat_agent.system_prompt,
            Some("You are a helpful test assistant.".to_string())
        );
    }

    #[test]
    fn test_simple_chat_agent_default_prompt() {
        let pool = create_test_pool();
        let mut conn = pool.get().unwrap();
        let id = uuid::Uuid::new_v4().to_string();

        // Create agent without system prompt
        let new_agent = NewAgent {
            id: id.clone(),
            name: "NoPromptAgent".to_string(),
            description: None,
            status: "active".to_string(),
            personality: None,
            tone: None,
            expertise: None,
            ai_provider: "gemini".to_string(),
            ai_model: "gemini-pro".to_string(),
            ai_temperature: 0.5,
            ai_config: "{}".to_string(),
            system_prompt: None, // No system prompt
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

        diesel::insert_into(agents::table)
            .values(&new_agent)
            .execute(&mut conn)
            .unwrap();

        let db_agent: DbAgent = agents::table
            .filter(agents::id.eq(id))
            .first(&mut conn)
            .unwrap();

        let chat_agent = SimpleChatAgent::new(&db_agent);

        assert_eq!(chat_agent.system_prompt, None);
        assert_eq!(chat_agent.model, "gemini-pro");
        assert_eq!(chat_agent.provider, "gemini");
    }

    #[test]
    fn test_load_history_empty_session() {
        let pool = create_test_pool();
        let db_agent = create_test_db_agent(&pool);
        let chat_agent = SimpleChatAgent::new(&db_agent);

        let history = chat_agent.load_history("nonexistent-session", &pool);

        assert!(history.is_empty());
    }

    #[test]
    fn test_save_and_load_message() {
        let pool = create_test_pool();
        let db_agent = create_test_db_agent(&pool);
        let chat_agent = SimpleChatAgent::new(&db_agent);
        let session_id = uuid::Uuid::new_v4().to_string();

        // Save some messages
        chat_agent.save_message(&pool, "user", "Hello!", &session_id);
        chat_agent.save_message(&pool, "assistant", "Hi there! How can I help?", &session_id);
        chat_agent.save_message(&pool, "user", "What is Rust?", &session_id);

        // Load and verify
        let history = chat_agent.load_history(&session_id, &pool);

        assert_eq!(history.len(), 3);
        // First message should be user
        assert!(matches!(history[0], Message::User { .. }));
        // Second should be assistant
        assert!(matches!(history[1], Message::Assistant { .. }));
        // Third should be user
        assert!(matches!(history[2], Message::User { .. }));
    }

    #[test]
    fn test_history_limit() {
        let pool = create_test_pool();
        let db_agent = create_test_db_agent(&pool);
        let chat_agent = SimpleChatAgent::new(&db_agent);
        let session_id = uuid::Uuid::new_v4().to_string();

        // Save more than the limit (20) messages
        for i in 0..30 {
            let role = if i % 2 == 0 { "user" } else { "assistant" };
            chat_agent.save_message(&pool, role, &format!("Message {}", i), &session_id);
        }

        // Load and verify limit
        let history = chat_agent.load_history(&session_id, &pool);

        // Should be limited to 20 messages
        assert_eq!(history.len(), 20);
    }
}
