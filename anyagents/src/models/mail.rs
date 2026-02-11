use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::mail_threads)]
pub struct MailThread {
    pub id: String,
    pub subject: String,
    pub is_read: i32,
    pub is_archived: i32,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::mail_threads)]
pub struct NewMailThread {
    pub id: String,
    pub subject: String,
    pub is_read: i32,
    pub is_archived: i32,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::mail_messages)]
pub struct MailMessage {
    pub id: String,
    pub thread_id: String,
    pub sender_type: String,
    pub sender_agent_id: Option<String>,
    pub recipient_type: String,
    pub recipient_agent_id: Option<String>,
    pub content: String,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::mail_messages)]
pub struct NewMailMessage {
    pub id: String,
    pub thread_id: String,
    pub sender_type: String,
    pub sender_agent_id: Option<String>,
    pub recipient_type: String,
    pub recipient_agent_id: Option<String>,
    pub content: String,
    pub created_at: chrono::NaiveDateTime,
}
