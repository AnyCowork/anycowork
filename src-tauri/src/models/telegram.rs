use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::telegram_configs)]
pub struct TelegramConfig {
    pub id: String,
    pub bot_token: String,
    pub agent_id: String,
    pub is_active: i32,
    pub allowed_chat_ids: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::telegram_configs)]
pub struct NewTelegramConfig {
    pub id: String,
    pub bot_token: String,
    pub agent_id: String,
    pub is_active: i32,
    pub allowed_chat_ids: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(AsChangeset, Deserialize)]
#[diesel(table_name = crate::schema::telegram_configs)]
pub struct UpdateTelegramConfig {
    pub bot_token: Option<String>,
    pub agent_id: Option<String>,
    pub is_active: Option<i32>,
    pub allowed_chat_ids: Option<String>,
    pub updated_at: chrono::NaiveDateTime,
}
