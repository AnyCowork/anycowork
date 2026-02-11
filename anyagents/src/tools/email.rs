use super::{Tool, ToolContext};
use crate::database::DbPool;
use crate::events::AgentObserver;
use crate::models::{Agent, NewMailMessage, NewMailThread};
use async_trait::async_trait;
use diesel::prelude::*;
use serde_json::{json, Value};

/// No-op observer for background agent tasks (no UI to emit to)
pub struct NoOpObserver;

impl AgentObserver for NoOpObserver {
    fn emit(&self, _event: &str, _payload: Value) -> Result<(), String> {
        Ok(())
    }
}

pub struct SendEmailTool {
    db_pool: DbPool,
    agent_id: String,
    agent_name: String,
    colleagues_description: String,
}

impl SendEmailTool {
    pub fn new(db_pool: DbPool, agent_id: String, agent_name: String, all_agents: Vec<Agent>) -> Self {
        let colleagues: Vec<String> = all_agents
            .iter()
            .filter(|a| a.id != agent_id)
            .map(|a| {
                let avatar = a.avatar.as_deref().unwrap_or("");
                let desc = a.description.as_deref().unwrap_or("");
                format!("{} {} ({})", avatar, a.name, desc)
            })
            .collect();

        let colleagues_description = if colleagues.is_empty() {
            "No colleagues available.".to_string()
        } else {
            format!("Your colleagues: {}. You can also send to 'user' for the user's mailbox.", colleagues.join(", "))
        };

        Self {
            db_pool,
            agent_id,
            agent_name,
            colleagues_description,
        }
    }
}

#[async_trait]
impl Tool for SendEmailTool {
    fn name(&self) -> &str {
        "send_email"
    }

    fn description(&self) -> &str {
        "Send an INTERNAL email to a colleague or to the user's mailbox. This is an internal messaging system - you do NOT need external email addresses. Just use the person's name (e.g., 'Jordan', 'Jordan the PM', or 'user'). Use list_colleagues tool first if you need to see who's available."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "to": {
                    "type": "string",
                    "description": format!("Recipient's NAME (not email address). Just use their name like 'Jordan' or 'Alex'. Use 'user' for the workspace owner. Available colleagues: {}", self.colleagues_description)
                },
                "subject": {
                    "type": "string",
                    "description": "Email subject line"
                },
                "body": {
                    "type": "string",
                    "description": "Email body content"
                }
            },
            "required": ["to", "subject", "body"]
        })
    }

    async fn execute(&self, args: Value, _ctx: &ToolContext) -> Result<Value, String> {
        let to = args["to"].as_str().ok_or("Missing 'to' field")?;
        let subject = args["subject"].as_str().ok_or("Missing 'subject' field")?;
        let body = args["body"].as_str().ok_or("Missing 'body' field")?;

        let now = chrono::Utc::now().naive_utc();

        // Determine recipient
        let (recipient_type, recipient_agent_id, _recipient_name) = if to.to_lowercase() == "user" {
            ("user".to_string(), None, "User".to_string())
        } else {
            // Look up agent by name (case-insensitive partial match)
            let mut conn = self.db_pool.get().map_err(|e| format!("DB error: {}", e))?;
            use crate::schema::agents::dsl::*;
            let all: Vec<Agent> = agents.load::<Agent>(&mut conn).map_err(|e| format!("DB error: {}", e))?;

            let target = all.iter().find(|a| {
                a.name.to_lowercase().contains(&to.to_lowercase())
                    || to.to_lowercase().contains(&a.name.to_lowercase())
            });

            match target {
                Some(agent) => ("agent".to_string(), Some(agent.id.clone()), agent.name.clone()),
                None => return Err(format!("Recipient '{}' not found. Available: {}", to, self.colleagues_description)),
            }
        };

        // Create mail thread
        let thread_id = uuid::Uuid::new_v4().to_string();
        let message_id = uuid::Uuid::new_v4().to_string();

        {
            let mut conn = self.db_pool.get().map_err(|e| format!("DB error: {}", e))?;

            let new_thread = NewMailThread {
                id: thread_id.clone(),
                subject: subject.to_string(),
                is_read: 0,
                is_archived: 0,
                created_at: now,
                updated_at: now,
            };

            diesel::insert_into(crate::schema::mail_threads::table)
                .values(&new_thread)
                .execute(&mut conn)
                .map_err(|e| format!("Failed to create mail thread: {}", e))?;

            let new_message = NewMailMessage {
                id: message_id.clone(),
                thread_id: thread_id.clone(),
                sender_type: "agent".to_string(),
                sender_agent_id: Some(self.agent_id.clone()),
                recipient_type: recipient_type.clone(),
                recipient_agent_id: recipient_agent_id.clone(),
                content: body.to_string(),
                created_at: now,
            };

            diesel::insert_into(crate::schema::mail_messages::table)
                .values(&new_message)
                .execute(&mut conn)
                .map_err(|e| format!("Failed to create mail message: {}", e))?;
        }

        // If recipient is another agent, spawn background processing
        if recipient_type == "agent" {
            if let Some(ref target_agent_id) = recipient_agent_id {
                let db_pool = self.db_pool.clone();
                let target_id = target_agent_id.clone();
                let sender_name = self.agent_name.clone();
                let sender_id = self.agent_id.clone();
                let subject_clone = subject.to_string();
                let body_clone = body.to_string();
                let thread_id_clone = thread_id.clone();

                tokio::spawn(async move {
                    if let Err(e) = process_email_in_background(
                        db_pool,
                        target_id,
                        sender_name,
                        sender_id,
                        subject_clone,
                        body_clone,
                        thread_id_clone,
                    ).await {
                        log::error!("Background email processing failed: {}", e);
                    }
                });
            }
        }

        Ok(json!({
            "status": "sent",
            "thread_id": thread_id,
            "message": format!("Email sent to {}", to)
        }))
    }
}

/// Process an email in the background using a direct LLM call (no tools/agent loop).
/// This prevents the agent from using tools or giving meta-responses.
pub async fn process_email_in_background(
    db_pool: DbPool,
    target_agent_id: String,
    sender_name: String,
    sender_agent_id: String,
    subject: String,
    body: String,
    thread_id: String,
) -> Result<(), String> {
    use crate::schema::agents::dsl::*;

    // 1. Load recipient agent
    let agent_db = {
        let mut conn = db_pool.get().map_err(|e| format!("DB error: {}", e))?;
        agents
            .filter(id.eq(&target_agent_id))
            .first::<Agent>(&mut conn)
            .map_err(|e| format!("Agent not found: {}", e))?
    };

    // 2. Build system prompt incorporating the agent's personality
    // IMPORTANT: Instruct the agent to write ONLY the reply content, no tool usage
    let preamble = format!(
        "{}\n\n---\n\nYou are composing an email reply. Write ONLY the reply content — no subject line, no meta-commentary, no tool usage, no JSON. Write naturally as yourself, addressing the request directly as if writing an email.",
        agent_db.system_prompt.as_deref().unwrap_or("You are a helpful assistant.")
    );

    // 3. Direct LLM call — no tools, no agent loop
    let client = crate::llm::LlmClient::new(&agent_db.ai_provider, &agent_db.ai_model)
        .with_preamble(&preamble);

    let prompt = format!(
        "You received an email from {sender_name}.\n\nSubject: {subject}\n\n{body}\n\nWrite your reply:",
    );

    let reply = client.prompt(&prompt).await.map_err(|e| format!("LLM error: {}", e))?;

    // Clean up the reply - remove any tool calls or meta-commentary
    let clean_reply = clean_email_reply(&reply);

    // 4. Save reply as a mail_message in the thread
    if !clean_reply.trim().is_empty() {
        let now = chrono::Utc::now().naive_utc();
        let mut conn = db_pool.get().map_err(|e| format!("DB error: {}", e))?;

        let new_reply = NewMailMessage {
            id: uuid::Uuid::new_v4().to_string(),
            thread_id: thread_id.clone(),
            sender_type: "agent".to_string(),
            sender_agent_id: Some(target_agent_id.clone()),
            recipient_type: "agent".to_string(),
            recipient_agent_id: Some(sender_agent_id),
            content: clean_reply.trim().to_string(),
            created_at: now,
        };

        diesel::insert_into(crate::schema::mail_messages::table)
            .values(&new_reply)
            .execute(&mut conn)
            .map_err(|e| format!("Failed to save reply: {}", e))?;

        // Update thread timestamp
        diesel::update(crate::schema::mail_threads::table.filter(crate::schema::mail_threads::id.eq(&thread_id)))
            .set(crate::schema::mail_threads::updated_at.eq(now))
            .execute(&mut conn)
            .map_err(|e| format!("Failed to update thread: {}", e))?;
    }

    log::info!("Background email processing completed for thread {}", thread_id);
    Ok(())
}

/// Clean email reply by removing any tool calls or JSON that might have been generated
fn clean_email_reply(reply: &str) -> String {
    let mut cleaned = reply.trim().to_string();

    // Remove any JSON blocks (tool calls)
    if let Some(json_start) = cleaned.find('{') {
        // Check if it looks like a tool call
        if cleaned[json_start..].contains("\"tool\"") {
            // Remove everything from the JSON onwards
            cleaned = cleaned[..json_start].trim().to_string();
        }
    }

    // Remove markdown code blocks if present
    if cleaned.contains("```json") || cleaned.contains("```") {
        // Extract just the text before code blocks
        if let Some(code_start) = cleaned.find("```") {
            cleaned = cleaned[..code_start].trim().to_string();
        }
    }

    // If the reply is empty after cleaning, return a fallback
    if cleaned.is_empty() {
        cleaned = "Thank you for your message. I've received it.".to_string();
    }

    cleaned
}
