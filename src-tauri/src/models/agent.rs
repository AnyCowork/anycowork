use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::agents)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub personality: Option<String>,
    pub tone: Option<String>,
    pub expertise: Option<String>,
    pub ai_provider: String,
    pub ai_model: String,
    pub ai_temperature: f32,
    pub ai_config: String,
    pub system_prompt: Option<String>,
    pub permissions: Option<String>,
    pub working_directories: Option<String>,
    pub skills: Option<String>,
    pub mcp_servers: Option<String>,
    pub messaging_connections: Option<String>,
    pub knowledge_bases: Option<String>,
    pub api_keys: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub platform_configs: Option<String>,
    pub execution_settings: Option<String>,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::agents)]
pub struct NewAgent {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub personality: Option<String>,
    pub tone: Option<String>,
    pub expertise: Option<String>,
    pub ai_provider: String,
    pub ai_model: String,
    pub ai_temperature: f32,
    pub ai_config: String,
    pub system_prompt: Option<String>,
    pub permissions: Option<String>,
    pub working_directories: Option<String>,
    pub skills: Option<String>,
    pub mcp_servers: Option<String>,
    pub messaging_connections: Option<String>,
    pub knowledge_bases: Option<String>,
    pub api_keys: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub platform_configs: Option<String>,
    pub execution_settings: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AgentCharacteristicsDto {
    pub personality: Option<String>,
    pub tone: Option<String>,
    pub expertise: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AIConfigDto {
    pub provider: String,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AgentDto {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub characteristics: AgentCharacteristicsDto,
    pub ai_config: AIConfigDto,
    pub system_prompt: Option<String>,
    pub skills: Vec<String>,
    pub mcp_servers: Vec<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct AgentUpdateDto {
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub characteristics: Option<AgentCharacteristicsDto>,
    pub ai_config: Option<AIConfigDto>,
    pub system_prompt: Option<String>,
    pub skills: Option<Vec<String>>,
    pub mcp_servers: Option<Vec<String>>,
    pub execution_settings: Option<serde_json::Value>,
}

impl Agent {
    pub fn into_dto(self) -> AgentDto {
        let characteristics = AgentCharacteristicsDto {
            personality: self.personality,
            tone: self.tone,
            expertise: self.expertise
                .map(|e| e.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
                .unwrap_or_default(),
        };

        let ai_config: AIConfigDto = serde_json::from_str(&self.ai_config).unwrap_or(AIConfigDto {
            provider: self.ai_provider,
            model: self.ai_model,
            temperature: self.ai_temperature,
            max_tokens: Some(4096),
        });

        AgentDto {
            id: self.id,
            name: self.name,
            description: self.description,
            status: self.status,
            characteristics,
            ai_config,
            system_prompt: self.system_prompt,
            skills: self.skills
                .map(|s| s.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
                .unwrap_or_default(),
            mcp_servers: self.mcp_servers
                .map(|s| s.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
                .unwrap_or_default(),
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}
