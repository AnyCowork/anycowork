use anyagents::models::{Agent, MailMessage, MailThread, NewMailMessage, NewMailThread};
use anyagents::schema;
use crate::AppState;
use diesel::prelude::*;
use serde::Serialize;
use tauri::State;


#[derive(Serialize, Clone, Debug)]
pub struct MailThreadWithPreview {
    pub id: String,
    pub subject: String,
    pub is_read: i32,
    pub is_archived: i32,
    pub created_at: String,
    pub updated_at: String,
    pub last_message_preview: Option<String>,
    pub last_sender_name: Option<String>,
    pub last_sender_avatar: Option<String>,
    pub message_count: i64,
}

#[derive(Serialize, Clone, Debug)]
pub struct MailMessageWithSender {
    pub id: String,
    pub thread_id: String,
    pub sender_type: String,
    pub sender_agent_id: Option<String>,
    pub sender_name: Option<String>,
    pub sender_avatar: Option<String>,
    pub recipient_type: String,
    pub recipient_agent_id: Option<String>,
    pub content: String,
    pub created_at: String,
}

#[tauri::command]
pub async fn get_mail_threads(
    state: State<'_, AppState>,
    account_id: Option<String>,
    folder: Option<String>,
    is_archived: Option<bool>,
) -> Result<Vec<MailThreadWithPreview>, String> {
    use schema::mail_messages;
    use schema::mail_threads;

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    // Get all threads
    let mut threads_query = mail_threads::table.into_boxed();

    if let Some(archived) = is_archived {
        threads_query = threads_query.filter(mail_threads::is_archived.eq(if archived { 1 } else { 0 }));
    } else {
        threads_query = threads_query.filter(mail_threads::is_archived.eq(0));
    }

    threads_query = threads_query.order(mail_threads::updated_at.desc());

    let all_threads: Vec<MailThread> = threads_query
        .load::<MailThread>(&mut conn)
        .map_err(|e| e.to_string())?;

    let folder_str = folder.unwrap_or_else(|| "inbox".to_string());

    // Load all agents for name/avatar lookup
    let all_agents: Vec<Agent> = schema::agents::table
        .load::<Agent>(&mut conn)
        .map_err(|e| e.to_string())?;

    let mut result = Vec::new();

    for thread in all_threads {
        // Get messages for this thread
        let msgs: Vec<MailMessage> = mail_messages::table
            .filter(mail_messages::thread_id.eq(&thread.id))
            .order(mail_messages::created_at.asc())
            .load::<MailMessage>(&mut conn)
            .map_err(|e| e.to_string())?;

        if msgs.is_empty() {
            continue;
        }

        // Filter by folder and account
        let thread_matches = match folder_str.as_str() {
            "sent" => {
                // Account is a sender in at least one message
                msgs.iter().any(|m| match &account_id {
                    None => m.sender_type == "user" && m.sender_agent_id.is_none(),
                    Some(aid) => m.sender_agent_id.as_ref() == Some(aid),
                })
            }
            _ => {
                // inbox: account is a recipient in at least one message
                msgs.iter().any(|m| match &account_id {
                    None => m.recipient_type == "user" && m.recipient_agent_id.is_none(),
                    Some(aid) => m.recipient_agent_id.as_ref() == Some(aid),
                })
            }
        };

        if !thread_matches {
            continue;
        }

        // Get last message for preview
        let last_msg = msgs.last().unwrap();
        let preview = last_msg.content.chars().take(100).collect::<String>();

        let sender_agent = last_msg.sender_agent_id.as_ref().and_then(|sid| {
            all_agents.iter().find(|a| &a.id == sid)
        });

        let sender_name = if last_msg.sender_type == "user" {
            Some("You".to_string())
        } else {
            sender_agent.map(|a| a.name.clone())
        };

        let sender_avatar = sender_agent.and_then(|a| a.avatar.clone());

        result.push(MailThreadWithPreview {
            id: thread.id,
            subject: thread.subject,
            is_read: thread.is_read,
            is_archived: thread.is_archived,
            created_at: thread.created_at.to_string(),
            updated_at: thread.updated_at.to_string(),
            last_message_preview: Some(preview),
            last_sender_name: sender_name,
            last_sender_avatar: sender_avatar,
            message_count: msgs.len() as i64,
        });
    }

    Ok(result)
}

#[tauri::command]
pub async fn get_mail_thread_messages(
    state: State<'_, AppState>,
    thread_id: String,
) -> Result<Vec<MailMessageWithSender>, String> {
    use schema::mail_messages;

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    let msgs: Vec<MailMessage> = mail_messages::table
        .filter(mail_messages::thread_id.eq(&thread_id))
        .order(mail_messages::created_at.asc())
        .load::<MailMessage>(&mut conn)
        .map_err(|e| e.to_string())?;

    let all_agents: Vec<Agent> = schema::agents::table
        .load::<Agent>(&mut conn)
        .map_err(|e| e.to_string())?;

    let result: Vec<MailMessageWithSender> = msgs
        .into_iter()
        .map(|m| {
            let sender_agent = m.sender_agent_id.as_ref().and_then(|sid| {
                all_agents.iter().find(|a| &a.id == sid)
            });

            let sender_name = if m.sender_type == "user" {
                Some("You".to_string())
            } else {
                sender_agent.map(|a| a.name.clone())
            };

            let sender_avatar = if m.sender_type == "user" {
                None
            } else {
                sender_agent.and_then(|a| a.avatar.clone())
            };

            MailMessageWithSender {
                id: m.id,
                thread_id: m.thread_id,
                sender_type: m.sender_type,
                sender_agent_id: m.sender_agent_id,
                sender_name,
                sender_avatar,
                recipient_type: m.recipient_type,
                recipient_agent_id: m.recipient_agent_id,
                content: m.content,
                created_at: m.created_at.to_string(),
            }
        })
        .collect();

    Ok(result)
}

#[tauri::command]
pub async fn send_mail(
    state: State<'_, AppState>,
    from_agent_id: Option<String>,
    to_agent_id: Option<String>,
    subject: String,
    body: String,
) -> Result<MailThreadWithPreview, String> {
    use schema::mail_messages;
    use schema::mail_threads;

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().naive_utc();

    let thread_id = uuid::Uuid::new_v4().to_string();

    let new_thread = NewMailThread {
        id: thread_id.clone(),
        subject: subject.clone(),
        is_read: 0,
        is_archived: 0,
        created_at: now,
        updated_at: now,
    };

    diesel::insert_into(mail_threads::table)
        .values(&new_thread)
        .execute(&mut conn)
        .map_err(|e| e.to_string())?;

    let sender_type = if from_agent_id.is_some() { "agent" } else { "user" };
    let recipient_type = if to_agent_id.is_some() { "agent" } else { "user" };

    let new_message = NewMailMessage {
        id: uuid::Uuid::new_v4().to_string(),
        thread_id: thread_id.clone(),
        sender_type: sender_type.to_string(),
        sender_agent_id: from_agent_id.clone(),
        recipient_type: recipient_type.to_string(),
        recipient_agent_id: to_agent_id.clone(),
        content: body.clone(),
        created_at: now,
    };

    diesel::insert_into(mail_messages::table)
        .values(&new_message)
        .execute(&mut conn)
        .map_err(|e| e.to_string())?;

    // If recipient is an agent, process in background
    if let Some(ref target_agent_id) = to_agent_id {
        let db_pool = state.db_pool.clone();
        let target_id = target_agent_id.clone();
        let sender_name = if let Some(ref fid) = from_agent_id {
            let agent: Option<Agent> = schema::agents::table
                .filter(schema::agents::id.eq(fid))
                .first::<Agent>(&mut conn)
                .ok();
            agent.map(|a| a.name).unwrap_or_else(|| "Unknown".to_string())
        } else {
            "User".to_string()
        };
        let sender_id_for_reply = from_agent_id.clone();
        let subject_clone = subject.clone();
        let body_clone = body.clone();
        let thread_id_clone = thread_id.clone();

        tauri::async_runtime::spawn(async move {
            if let Err(e) = process_mail_background(
                db_pool,
                target_id,
                sender_name,
                sender_id_for_reply,
                subject_clone,
                body_clone,
                thread_id_clone,
            ).await {
                log::error!("Background mail processing failed: {}", e);
            }
        });
    }

    // Get sender info for preview
    let all_agents: Vec<Agent> = schema::agents::table
        .load::<Agent>(&mut conn)
        .map_err(|e| e.to_string())?;

    let sender_agent = from_agent_id.as_ref().and_then(|fid| {
        all_agents.iter().find(|a| &a.id == fid)
    });

    Ok(MailThreadWithPreview {
        id: thread_id,
        subject,
        is_read: 0,
        is_archived: 0,
        created_at: now.to_string(),
        updated_at: now.to_string(),
        last_message_preview: Some(body.chars().take(100).collect()),
        last_sender_name: if from_agent_id.is_some() {
            sender_agent.map(|a| a.name.clone())
        } else {
            Some("You".to_string())
        },
        last_sender_avatar: sender_agent.and_then(|a| a.avatar.clone()),
        message_count: 1,
    })
}

#[tauri::command]
pub async fn reply_to_mail(
    state: State<'_, AppState>,
    thread_id: String,
    from_agent_id: Option<String>,
    content: String,
) -> Result<MailMessageWithSender, String> {
    use schema::mail_messages;
    use schema::mail_threads;

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().naive_utc();

    // Get existing messages to determine the "other side"
    let existing_msgs: Vec<MailMessage> = mail_messages::table
        .filter(mail_messages::thread_id.eq(&thread_id))
        .order(mail_messages::created_at.desc())
        .load::<MailMessage>(&mut conn)
        .map_err(|e| e.to_string())?;

    // Determine recipient (the "other side" of the conversation)
    let (recipient_type, recipient_agent_id) = if let Some(last_msg) = existing_msgs.first() {
        // Reply to the last sender (unless it's us)
        let is_self = match &from_agent_id {
            None => last_msg.sender_type == "user" && last_msg.sender_agent_id.is_none(),
            Some(fid) => last_msg.sender_agent_id.as_ref() == Some(fid),
        };

        if is_self {
            // We are replying to ourselves? Use the original recipient
            (last_msg.recipient_type.clone(), last_msg.recipient_agent_id.clone())
        } else {
            (last_msg.sender_type.clone(), last_msg.sender_agent_id.clone())
        }
    } else {
        return Err("Thread has no messages".to_string());
    };

    let sender_type = if from_agent_id.is_some() { "agent" } else { "user" };

    let message_id = uuid::Uuid::new_v4().to_string();
    let new_message = NewMailMessage {
        id: message_id.clone(),
        thread_id: thread_id.clone(),
        sender_type: sender_type.to_string(),
        sender_agent_id: from_agent_id.clone(),
        recipient_type: recipient_type.clone(),
        recipient_agent_id: recipient_agent_id.clone(),
        content: content.clone(),
        created_at: now,
    };

    diesel::insert_into(mail_messages::table)
        .values(&new_message)
        .execute(&mut conn)
        .map_err(|e| e.to_string())?;

    // Update thread timestamp and mark unread
    diesel::update(mail_threads::table.filter(mail_threads::id.eq(&thread_id)))
        .set((
            mail_threads::updated_at.eq(now),
            mail_threads::is_read.eq(0),
        ))
        .execute(&mut conn)
        .map_err(|e| e.to_string())?;

    // If recipient is an agent, process in background
    if recipient_type == "agent" {
        if let Some(ref target_agent_id) = recipient_agent_id {
            let db_pool = state.db_pool.clone();
            let target_id = target_agent_id.clone();
            let sender_name = if let Some(ref fid) = from_agent_id {
                let agent: Option<Agent> = schema::agents::table
                    .filter(schema::agents::id.eq(fid))
                    .first::<Agent>(&mut conn)
                    .ok();
                agent.map(|a| a.name).unwrap_or_else(|| "Unknown".to_string())
            } else {
                "User".to_string()
            };
            let sender_id_for_reply = from_agent_id.clone();

            // Get thread subject
            let thread: MailThread = mail_threads::table
                .filter(mail_threads::id.eq(&thread_id))
                .first::<MailThread>(&mut conn)
                .map_err(|e| e.to_string())?;

            let thread_id_clone = thread_id.clone();
            let content_clone = content.clone();

            tauri::async_runtime::spawn(async move {
                if let Err(e) = process_mail_background(
                    db_pool,
                    target_id,
                    sender_name,
                    sender_id_for_reply,
                    thread.subject,
                    content_clone,
                    thread_id_clone,
                ).await {
                    log::error!("Background reply processing failed: {}", e);
                }
            });
        }
    }

    // Build response with sender info
    let all_agents: Vec<Agent> = schema::agents::table
        .load::<Agent>(&mut conn)
        .map_err(|e| e.to_string())?;

    let sender_agent = from_agent_id.as_ref().and_then(|fid| {
        all_agents.iter().find(|a| &a.id == fid)
    });

    Ok(MailMessageWithSender {
        id: message_id,
        thread_id,
        sender_type: sender_type.to_string(),
        sender_agent_id: from_agent_id,
        sender_name: if sender_type == "user" {
            Some("You".to_string())
        } else {
            sender_agent.map(|a| a.name.clone())
        },
        sender_avatar: sender_agent.and_then(|a| a.avatar.clone()),
        recipient_type,
        recipient_agent_id,
        content,
        created_at: now.to_string(),
    })
}

#[tauri::command]
pub async fn mark_thread_read(
    state: State<'_, AppState>,
    thread_id: String,
) -> Result<(), String> {
    use schema::mail_threads;

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    diesel::update(mail_threads::table.filter(mail_threads::id.eq(&thread_id)))
        .set(mail_threads::is_read.eq(1))
        .execute(&mut conn)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn archive_thread(
    state: State<'_, AppState>,
    thread_id: String,
) -> Result<(), String> {
    use schema::mail_threads;

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    diesel::update(mail_threads::table.filter(mail_threads::id.eq(&thread_id)))
        .set(mail_threads::is_archived.eq(1))
        .execute(&mut conn)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn get_unread_mail_count(
    state: State<'_, AppState>,
    account_id: Option<String>,
) -> Result<i64, String> {
    use schema::mail_messages;
    use schema::mail_threads;

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    // Get unread, non-archived threads
    let unread_threads: Vec<MailThread> = mail_threads::table
        .filter(mail_threads::is_read.eq(0))
        .filter(mail_threads::is_archived.eq(0))
        .load::<MailThread>(&mut conn)
        .map_err(|e| e.to_string())?;

    let mut count = 0i64;
    for thread in unread_threads {
        let msgs: Vec<MailMessage> = mail_messages::table
            .filter(mail_messages::thread_id.eq(&thread.id))
            .load::<MailMessage>(&mut conn)
            .map_err(|e| e.to_string())?;

        let is_inbox = msgs.iter().any(|m| match &account_id {
            None => m.recipient_type == "user" && m.recipient_agent_id.is_none(),
            Some(aid) => m.recipient_agent_id.as_ref() == Some(aid),
        });

        if is_inbox {
            count += 1;
        }
    }

    Ok(count)
}

/// Background processing: agent reads email and composes a reply
/// Uses direct LLM call without tools to prevent meta-responses
async fn process_mail_background(
    db_pool: anyagents::database::DbPool,
    target_agent_id: String,
    _sender_name: String,
    sender_agent_id: Option<String>,
    subject: String,
    _body: String,
    thread_id: String,
) -> Result<(), String> {
    use schema::agents::dsl::*;

    // 1. Load data from DB (Agent + Thread History)
    let (agent_db, conversation_history) = {
        let mut conn = db_pool.get().map_err(|e| format!("DB error: {}", e))?;
        
        // Load agent
        let agent = agents
            .filter(id.eq(&target_agent_id))
            .first::<Agent>(&mut conn)
            .map_err(|e| format!("Agent not found: {}", e))?;

        // Load thread history
        use schema::mail_messages;
        let thread_messages: Vec<MailMessage> = mail_messages::table
            .filter(mail_messages::thread_id.eq(&thread_id))
            .order(mail_messages::created_at.asc())
            .load::<MailMessage>(&mut conn)
            .map_err(|e| format!("Failed to load thread history: {}", e))?;

        // Format history
        let mut history = String::new();
        for msg in thread_messages {
            let name_label = if msg.sender_agent_id == Some(target_agent_id.clone()) {
                "You".to_string()
            } else if msg.sender_type == "user" {
                "User".to_string()
            } else {
                "Sender".to_string()
            };

            history.push_str(&format!("{}: {}\n\n", name_label, msg.content));
        }
        
        (agent, history)
    };

    // 2. Build system prompt incorporating the agent's personality
    // IMPORTANT: Instruct the agent to write ONLY the reply content, no tool usage
    let preamble = format!(
        "{}\n\n---\n\nYou are composing an email reply. Write ONLY the reply content — no subject line, no meta-commentary, no tool usage, no JSON. Write naturally as yourself, addressing the request directly as if writing an email.",
        agent_db.system_prompt.as_deref().unwrap_or("You are a helpful assistant.")
    );

    // 4. LLM call
    let key_name = match agent_db.ai_provider.as_str() {
        "openai" => "OPENAI_API_KEY",
        "gemini" => "GEMINI_API_KEY",
        "anthropic" => "ANTHROPIC_API_KEY",
        _ => "",
    };
    let api_key = anyagents::models::settings::get_setting(&db_pool, key_name);

    let mut client = anyagents::llm::LlmClient::new(&agent_db.ai_provider, &agent_db.ai_model)
        .with_preamble(&preamble);
    
    if let Some(key) = api_key {
        client = client.with_api_key(&key);
    }

    let prompt = format!(
        "Subject: {subject}\n\nHere is the conversation history:\n\n{conversation_history}\n\nWrite your reply:",
    );

    let reply = client.prompt(&prompt).await.map_err(|e| format!("LLM error: {}", e))?;

    // 5. Save reply as a mail_message in the thread
    if !reply.trim().is_empty() {
        let now = chrono::Utc::now().naive_utc();
        let mut conn = db_pool.get().map_err(|e| format!("DB error: {}", e))?;

        // Reply goes to the original sender or to user if sender was user
        let (reply_recipient_type, reply_recipient_agent_id) = match &sender_agent_id {
            Some(sid) => ("agent".to_string(), Some(sid.clone())),
            None => ("user".to_string(), None),
        };

        // Clean up the reply - remove any tool calls or meta-commentary
        let clean_reply = clean_email_reply(&reply);

        let new_reply = NewMailMessage {
            id: uuid::Uuid::new_v4().to_string(),
            thread_id: thread_id.clone(),
            sender_type: "agent".to_string(),
            sender_agent_id: Some(target_agent_id.clone()),
            recipient_type: reply_recipient_type,
            recipient_agent_id: reply_recipient_agent_id,
            content: clean_reply,
            created_at: now,
        };

        diesel::insert_into(schema::mail_messages::table)
            .values(&new_reply)
            .execute(&mut conn)
            .map_err(|e| format!("Failed to save reply: {}", e))?;

        // Update thread timestamp and mark unread
        diesel::update(schema::mail_threads::table.filter(schema::mail_threads::id.eq(&thread_id)))
            .set((
                schema::mail_threads::updated_at.eq(now),
                schema::mail_threads::is_read.eq(0),
            ))
            .execute(&mut conn)
            .map_err(|e| format!("Failed to update thread: {}", e))?;
    }

    log::info!("Background mail processing completed for thread {}", thread_id);
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
