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
