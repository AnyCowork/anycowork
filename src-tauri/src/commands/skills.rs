use crate::models::{AgentSkill, NewAgentSkill, UpdateAgentSkill};
use crate::schema;
use crate::AppState;
use diesel::prelude::*;
use tauri::State;

#[tauri::command]
pub async fn get_skills(
    state: State<'_, AppState>,
    enabled_only: Option<bool>,
) -> Result<Vec<AgentSkill>, String> {
    use schema::agent_skills::dsl::{agent_skills, enabled, name};

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    let mut query = agent_skills.into_boxed();

    if let Some(true) = enabled_only {
        query = query.filter(enabled.eq(1));
    }

    query
        .order(name.asc())
        .load::<AgentSkill>(&mut conn)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_skill(state: State<'_, AppState>, skill_id: String) -> Result<AgentSkill, String> {
    use schema::agent_skills::dsl::{agent_skills, id};

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;
    agent_skills
        .filter(id.eq(skill_id))
        .first::<AgentSkill>(&mut conn)
        .map_err(|e| format!("Skill not found: {}", e))
}

#[tauri::command]
pub async fn create_skill(
    state: State<'_, AppState>,
    name_param: String,
    display_title: String,
    description: String,
    skill_content: String,
    additional_files_json: Option<String>,
) -> Result<AgentSkill, String> {
    use schema::agent_skills;

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    let new_skill = NewAgentSkill {
        id: uuid::Uuid::new_v4().to_string(),
        name: name_param,
        display_title,
        description,
        skill_content,
        additional_files_json,
        enabled: 1,
        version: 1,
        created_at: chrono::Utc::now().naive_utc(),
        updated_at: chrono::Utc::now().naive_utc(),
    };

    diesel::insert_into(agent_skills::table)
        .values(&new_skill)
        .execute(&mut conn)
        .map_err(|e| e.to_string())?;

    let created_id = new_skill.id.clone();

    agent_skills::table
        .filter(agent_skills::id.eq(created_id))
        .first::<AgentSkill>(&mut conn)
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn update_skill(
    state: State<'_, AppState>,
    skill_id: String,
    name_param: Option<String>,
    display_title: Option<String>,
    description: Option<String>,
    skill_content: Option<String>,
    additional_files_json: Option<String>,
    enabled_param: Option<bool>,
) -> Result<AgentSkill, String> {
    use schema::agent_skills::dsl::{agent_skills, id};

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    // Increment version on update
    let current_version: i32 = agent_skills
        .filter(id.eq(&skill_id))
        .select(schema::agent_skills::version)
        .first(&mut conn)
        .unwrap_or(1);

    let update = UpdateAgentSkill {
        name: name_param,
        display_title,
        description,
        skill_content,
        additional_files_json,
        enabled: enabled_param.map(|v| if v { 1 } else { 0 }),
        version: Some(current_version + 1),
        updated_at: chrono::Utc::now().naive_utc(),
    };

    diesel::update(agent_skills.filter(id.eq(&skill_id)))
        .set(&update)
        .execute(&mut conn)
        .map_err(|e| e.to_string())?;

    agent_skills
        .filter(id.eq(&skill_id))
        .first::<AgentSkill>(&mut conn)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_skill(state: State<'_, AppState>, skill_id: String) -> Result<(), String> {
    use schema::agent_skills::dsl::{agent_skills, id};

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;
    diesel::delete(agent_skills.filter(id.eq(skill_id)))
        .execute(&mut conn)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn toggle_skill(
    state: State<'_, AppState>,
    skill_id: String,
) -> Result<AgentSkill, String> {
    use schema::agent_skills::dsl::{agent_skills, enabled, id};

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    // Get current enabled status
    let current_enabled: i32 = agent_skills
        .filter(id.eq(&skill_id))
        .select(enabled)
        .first(&mut conn)
        .map_err(|e| format!("Skill not found: {}", e))?;

    let new_enabled = if current_enabled == 1 { 0 } else { 1 };

    let update = UpdateAgentSkill {
        name: None,
        display_title: None,
        description: None,
        skill_content: None,
        additional_files_json: None,
        enabled: Some(new_enabled),
        version: None,
        updated_at: chrono::Utc::now().naive_utc(),
    };

    diesel::update(agent_skills.filter(id.eq(&skill_id)))
        .set(&update)
        .execute(&mut conn)
        .map_err(|e| e.to_string())?;

    agent_skills
        .filter(id.eq(&skill_id))
        .first::<AgentSkill>(&mut conn)
        .map_err(|e| e.to_string())
}
