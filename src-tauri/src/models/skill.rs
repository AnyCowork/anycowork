use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::agent_skills)]
pub struct AgentSkill {
    pub id: String,
    pub name: String,
    pub display_title: String,
    pub description: String,
    pub skill_content: String,
    pub additional_files_json: Option<String>,
    pub enabled: i32,
    pub version: i32,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub source_path: Option<String>,
    pub category: Option<String>,
    pub requires_sandbox: i32,
    pub sandbox_config: Option<String>,
    pub execution_mode: String,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::agent_skills)]
pub struct NewAgentSkill {
    pub id: String,
    pub name: String,
    pub display_title: String,
    pub description: String,
    pub skill_content: String,
    pub additional_files_json: Option<String>,
    pub enabled: i32,
    pub version: i32,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub source_path: Option<String>,
    pub category: Option<String>,
    pub requires_sandbox: i32,
    pub sandbox_config: Option<String>,
    pub execution_mode: String,
}

#[derive(AsChangeset, Deserialize)]
#[diesel(table_name = crate::schema::agent_skills)]
pub struct UpdateAgentSkill {
    pub name: Option<String>,
    pub display_title: Option<String>,
    pub description: Option<String>,
    pub skill_content: Option<String>,
    pub additional_files_json: Option<String>,
    pub enabled: Option<i32>,
    pub version: Option<i32>,
    pub updated_at: chrono::NaiveDateTime,
    pub source_path: Option<String>,
    pub category: Option<String>,
    pub requires_sandbox: Option<i32>,
    pub sandbox_config: Option<String>,
    pub execution_mode: Option<String>,
}

// Skill file model for storing bundled files
#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::skill_files)]
pub struct SkillFile {
    pub id: String,
    pub skill_id: String,
    pub relative_path: String,
    pub content: String,
    pub file_type: String,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::skill_files)]
pub struct NewSkillFile {
    pub id: String,
    pub skill_id: String,
    pub relative_path: String,
    pub content: String,
    pub file_type: String,
    pub created_at: chrono::NaiveDateTime,
}

// DTO for marketplace skills
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MarketplaceSkill {
    pub id: String,
    pub name: String,
    pub display_title: String,
    pub description: String,
    pub category: Option<String>,
    pub dir_name: String,
    pub is_installed: bool,
}

// Parsed skill from SKILL.md
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedSkill {
    pub name: String,
    pub description: String,
    pub license: Option<String>,
    pub category: Option<String>,
    pub triggers: Option<Vec<String>>,
    pub requires_sandbox: bool,
    pub sandbox_config: Option<SandboxConfig>,
    pub execution_mode: Option<String>, // "sandbox", "direct", "flexible"
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    pub image: Option<String>,
    pub memory_limit: Option<String>,
    pub cpu_limit: Option<f32>,
    pub timeout_seconds: Option<u32>,
    pub network_enabled: Option<bool>,
}

#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::agent_skill_assignments)]
pub struct AgentSkillAssignment {
    pub agent_id: String,
    pub skill_id: String,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::agent_skill_assignments)]
pub struct NewAgentSkillAssignment {
    pub agent_id: String,
    pub skill_id: String,
    pub created_at: chrono::NaiveDateTime,
}
