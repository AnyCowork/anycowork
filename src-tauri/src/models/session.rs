use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::sessions)]
pub struct Session {
    pub id: String,
    pub agent_id: String,
    pub title: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub archived: i32,
    pub pinned: i32,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::sessions)]
pub struct NewSession {
    pub id: String,
    pub agent_id: String,
    pub title: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub archived: i32,
    pub pinned: i32,
}

#[derive(AsChangeset, Deserialize)]
#[diesel(table_name = crate::schema::sessions)]
pub struct UpdateSession {
    pub title: Option<String>,
    pub archived: Option<i32>,
    pub pinned: Option<i32>,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::messages)]
pub struct Message {
    pub id: String,
    pub role: String,
    pub content: String,
    pub session_id: String,
    pub created_at: chrono::NaiveDateTime,
    pub metadata_json: Option<String>,
    pub tokens: Option<i32>,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::messages)]
pub struct NewMessage {
    pub id: String,
    pub role: String,
    pub content: String,
    pub session_id: String,
    pub metadata_json: Option<String>,
    pub tokens: Option<i32>,
}
