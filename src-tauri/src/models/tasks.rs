use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::tasks)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: i32,
    pub session_id: Option<String>,
    pub agent_id: Option<String>,
    pub created_at: String, // Stored as string in SQLite for simplicity with current setup
    pub updated_at: String,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::tasks)]
pub struct NewTask {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: i32,
    pub session_id: Option<String>,
    pub agent_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(AsChangeset, Deserialize)]
#[diesel(table_name = crate::schema::tasks)]
pub struct UpdateTask {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub priority: Option<i32>,
    pub session_id: Option<String>, // Can assign to session
    pub updated_at: String,
}
