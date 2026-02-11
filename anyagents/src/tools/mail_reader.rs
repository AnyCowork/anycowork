use super::{Tool, ToolContext};
use crate::database::DbPool;
use crate::models::{Agent, MailMessage, MailThread};
use async_trait::async_trait;
use diesel::prelude::*;
use serde_json::{json, Value};

/// Tool to read and search mail messages
pub struct MailReaderTool {
    db_pool: DbPool,
    agent_id: String,
}

impl MailReaderTool {
    pub fn new(db_pool: DbPool, agent_id: String) -> Self {
        Self { db_pool, agent_id }
    }
}

#[async_trait]
impl Tool for MailReaderTool {
    fn name(&self) -> &str {
        "check_mail"
    }

    fn description(&self) -> &str {
        "Check your inbox or sent mail. Use this to:\n\
         - See unread messages\n\
         - Search for emails from specific people\n\
         - Check your sent emails\n\
         - Find email threads by subject\n\
         \n\
         This is YOUR internal mailbox. Use it to stay informed about communications."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "folder": {
                    "type": "string",
                    "enum": ["inbox", "sent", "all"],
                    "description": "Which folder to check: 'inbox' (default), 'sent', or 'all'",
                    "default": "inbox"
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of emails to return (default: 10)",
                    "default": 10,
                    "minimum": 1,
                    "maximum": 50
                },
                "search": {
                    "type": "string",
                    "description": "Optional: Search term to filter emails by subject or content"
                },
                "from": {
                    "type": "string",
                    "description": "Optional: Filter emails from specific sender (name or partial name)"
                },
                "unread_only": {
                    "type": "boolean",
                    "description": "Optional: Only show unread emails",
                    "default": false
                }
            },
            "required": []
        })
    }

    async fn execute(&self, args: Value, _ctx: &ToolContext) -> Result<Value, String> {
        use crate::schema::{mail_messages, mail_threads};

        let mut conn = self.db_pool.get().map_err(|e| format!("DB error: {}", e))?;

        let folder = args.get("folder")
            .and_then(|f| f.as_str())
            .unwrap_or("inbox");

        let limit = args.get("limit")
            .and_then(|l| l.as_i64())
            .unwrap_or(10) as i64;

        let search_term = args.get("search").and_then(|s| s.as_str());
        let from_filter = args.get("from").and_then(|f| f.as_str());
        let unread_only = args.get("unread_only")
            .and_then(|u| u.as_bool())
            .unwrap_or(false);

        // Get all threads first
        let mut threads_query = mail_threads::table.into_boxed();

        if unread_only {
            threads_query = threads_query.filter(mail_threads::is_read.eq(0));
        }

        if let Some(search) = search_term {
            threads_query = threads_query.filter(
                mail_threads::subject.like(format!("%{}%", search))
            );
        }

        threads_query = threads_query
            .order(mail_threads::updated_at.desc())
            .limit(limit);

        let threads: Vec<MailThread> = threads_query
            .load(&mut conn)
            .map_err(|e| format!("DB error: {}", e))?;

        // Get all agents for name resolution
        use crate::schema::agents;
        let all_agents: Vec<Agent> = agents::table
            .load(&mut conn)
            .map_err(|e| format!("DB error: {}", e))?;

        let mut results = Vec::new();

        for thread in threads {
            // Get messages for this thread
            let messages: Vec<MailMessage> = mail_messages::table
                .filter(mail_messages::thread_id.eq(&thread.id))
                .order(mail_messages::created_at.asc())
                .load(&mut conn)
                .map_err(|e| format!("DB error: {}", e))?;

            if messages.is_empty() {
                continue;
            }

            // Filter by folder
            let matches_folder = match folder {
                "inbox" => {
                    messages.iter().any(|m| {
                        m.recipient_agent_id.as_ref() == Some(&self.agent_id)
                    })
                }
                "sent" => {
                    messages.iter().any(|m| {
                        m.sender_agent_id.as_ref() == Some(&self.agent_id)
                    })
                }
                "all" => true,
                _ => true,
            };

            if !matches_folder {
                continue;
            }

            // Filter by sender if specified
            if let Some(from_name) = from_filter {
                let has_matching_sender = messages.iter().any(|m| {
                    if let Some(ref sender_id) = m.sender_agent_id {
                        all_agents.iter().any(|a| {
                            &a.id == sender_id &&
                            a.name.to_lowercase().contains(&from_name.to_lowercase())
                        })
                    } else {
                        m.sender_type == "user" &&
                        "user".to_lowercase().contains(&from_name.to_lowercase())
                    }
                });

                if !has_matching_sender {
                    continue;
                }
            }

            // Get last message for preview
            let last_msg = messages.last().unwrap();
            let sender_name = if last_msg.sender_type == "user" {
                "User".to_string()
            } else if let Some(ref sender_id) = last_msg.sender_agent_id {
                all_agents.iter()
                    .find(|a| &a.id == sender_id)
                    .map(|a| a.name.clone())
                    .unwrap_or_else(|| "Unknown".to_string())
            } else {
                "Unknown".to_string()
            };

            let preview = last_msg.content
                .chars()
                .take(100)
                .collect::<String>();

            results.push(json!({
                "thread_id": thread.id,
                "subject": thread.subject,
                "from": sender_name,
                "preview": preview,
                "message_count": messages.len(),
                "is_read": thread.is_read == 1,
                "updated_at": thread.updated_at.to_string(),
            }));
        }

        Ok(json!({
            "folder": folder,
            "emails": results,
            "count": results.len(),
            "note": if results.is_empty() {
                format!("No emails found in {}.", folder)
            } else {
                format!("Found {} emails in {}.", results.len(), folder)
            }
        }))
    }

    fn verify_result(&self, result: &Value) -> bool {
        result.get("emails").is_some()
    }
}

/// Tool to read a specific email thread
pub struct ReadEmailThreadTool {
    db_pool: DbPool,
}

impl ReadEmailThreadTool {
    pub fn new(db_pool: DbPool) -> Self {
        Self { db_pool }
    }
}

#[async_trait]
impl Tool for ReadEmailThreadTool {
    fn name(&self) -> &str {
        "read_email_thread"
    }

    fn description(&self) -> &str {
        "Read all messages in a specific email thread. Use this after check_mail to see the full conversation."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "thread_id": {
                    "type": "string",
                    "description": "The thread_id from check_mail results"
                }
            },
            "required": ["thread_id"]
        })
    }

    async fn execute(&self, args: Value, _ctx: &ToolContext) -> Result<Value, String> {
        use crate::schema::{mail_messages, mail_threads};

        let thread_id = args.get("thread_id")
            .and_then(|t| t.as_str())
            .ok_or("Missing thread_id")?;

        let mut conn = self.db_pool.get().map_err(|e| format!("DB error: {}", e))?;

        // Get thread
        let thread: MailThread = mail_threads::table
            .filter(mail_threads::id.eq(thread_id))
            .first(&mut conn)
            .map_err(|_| "Thread not found")?;

        // Get messages
        let messages: Vec<MailMessage> = mail_messages::table
            .filter(mail_messages::thread_id.eq(thread_id))
            .order(mail_messages::created_at.asc())
            .load(&mut conn)
            .map_err(|e| format!("DB error: {}", e))?;

        // Get all agents for name resolution
        use crate::schema::agents;
        let all_agents: Vec<Agent> = agents::table
            .load(&mut conn)
            .map_err(|e| format!("DB error: {}", e))?;

        let mut formatted_messages = Vec::new();

        for msg in messages {
            let sender_name = if msg.sender_type == "user" {
                "User".to_string()
            } else if let Some(ref sender_id) = msg.sender_agent_id {
                all_agents.iter()
                    .find(|a| &a.id == sender_id)
                    .map(|a| a.name.clone())
                    .unwrap_or_else(|| "Unknown".to_string())
            } else {
                "Unknown".to_string()
            };

            formatted_messages.push(json!({
                "from": sender_name,
                "content": msg.content,
                "timestamp": msg.created_at.to_string(),
            }));
        }

        Ok(json!({
            "subject": thread.subject,
            "messages": formatted_messages,
            "message_count": formatted_messages.len(),
        }))
    }
}
