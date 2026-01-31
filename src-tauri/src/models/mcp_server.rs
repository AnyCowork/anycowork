use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::mcp_servers)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct McpServer {
    pub id: String,
    pub name: String,
    pub server_type: String, // "stdio" or "sse"
    pub command: Option<String>,
    pub args: Option<String>,
    pub env: Option<String>,
    pub url: Option<String>,
    pub is_enabled: i32,
    pub template_id: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::mcp_servers)]
pub struct NewMcpServer {
    pub id: String,
    pub name: String,
    pub server_type: String,
    pub command: Option<String>,
    pub args: Option<String>,
    pub env: Option<String>,
    pub url: Option<String>,
    pub is_enabled: i32,
    pub template_id: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct McpServerDto {
    pub id: String,
    pub name: String,
    pub server_type: String,
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub env: Option<serde_json::Value>,
    pub url: Option<String>,
    pub is_enabled: bool,
    pub template_id: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct McpServerUpdateDto {
    pub name: Option<String>,
    pub server_type: Option<String>,
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub env: Option<serde_json::Value>,
    pub url: Option<String>,
    pub is_enabled: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct McpTemplateDto {
    pub id: String,
    pub name: String,
    pub description: String,
    pub server_type: String,
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub env: Option<serde_json::Value>,
    pub url: Option<String>,
}

impl McpServer {
    pub fn into_dto(self) -> McpServerDto {
        McpServerDto {
            id: self.id,
            name: self.name,
            server_type: self.server_type,
            command: self.command,
            args: self.args.and_then(|s| serde_json::from_str(&s).ok()),
            env: self.env.and_then(|s| serde_json::from_str(&s).ok()),
            url: self.url,
            is_enabled: self.is_enabled != 0,
            template_id: self.template_id,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}
