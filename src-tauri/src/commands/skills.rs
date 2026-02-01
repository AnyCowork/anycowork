use crate::models::{
    AgentSkill, AgentSkillAssignment, MarketplaceSkill, NewAgentSkill, NewAgentSkillAssignment,
    NewSkillFile, SkillFile, UpdateAgentSkill,
};
use crate::schema;
use crate::skills::docker::DockerSandbox;
use crate::skills::loader::{load_skill_from_directory, load_skill_from_zip, list_marketplace_skills as scan_marketplace_skills};
use crate::AppState;
use diesel::prelude::*;
use std::path::Path;
use tauri::State;

// ==================== BASIC CRUD ====================

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
        source_path: None,
        category: Some("General".to_string()),
        requires_sandbox: 0,
        sandbox_config: None,
        execution_mode: "direct".to_string(),
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
    category: Option<String>,
    requires_sandbox: Option<bool>,
    sandbox_config: Option<String>,
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
        source_path: None,
        category,
        requires_sandbox: requires_sandbox.map(|v| if v { 1 } else { 0 }),
        sandbox_config,
        execution_mode: None,
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

    // Delete skill files first (cascade should handle this, but explicit is safer)
    diesel::delete(schema::skill_files::table.filter(schema::skill_files::skill_id.eq(&skill_id)))
        .execute(&mut conn)
        .ok();

    // Delete skill assignments
    diesel::delete(
        schema::agent_skill_assignments::table
            .filter(schema::agent_skill_assignments::skill_id.eq(&skill_id)),
    )
    .execute(&mut conn)
    .ok();

    // Delete skill
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
        source_path: None,
        category: None,
        requires_sandbox: None,
        sandbox_config: None,
        execution_mode: None,
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

// ==================== IMPORT COMMANDS ====================

#[tauri::command]
pub async fn import_skill_from_directory(
    state: State<'_, AppState>,
    directory_path: String,
) -> Result<AgentSkill, String> {
    let path = Path::new(&directory_path);
    let loaded = load_skill_from_directory(path)?;

    save_loaded_skill(&state, loaded, Some(directory_path)).await
}

#[tauri::command]
pub async fn import_skill_from_zip(
    state: State<'_, AppState>,
    zip_path: String,
) -> Result<AgentSkill, String> {
    let path = Path::new(&zip_path);
    let loaded = load_skill_from_zip(path)?;

    save_loaded_skill(&state, loaded, Some(zip_path)).await
}

/// Helper function to save a loaded skill to the database
/// If a skill with the same name already exists, it will be updated instead of creating a duplicate
async fn save_loaded_skill(
    state: &State<'_, AppState>,
    loaded: crate::skills::loader::LoadedSkill,
    source_path: Option<String>,
) -> Result<AgentSkill, String> {
    use schema::agent_skills;

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    let skill = &loaded.skill;

    // Check if a skill with the same name already exists
    let existing_skill: Option<AgentSkill> = agent_skills::table
        .filter(agent_skills::name.eq(&skill.name))
        .first::<AgentSkill>(&mut conn)
        .ok();

    // Serialize sandbox config if present
    let sandbox_config_json = skill
        .sandbox_config
        .as_ref()
        .map(|c| serde_json::to_string(c).unwrap_or_default());

    let skill_id = if let Some(existing) = existing_skill {
        // Update existing skill instead of creating duplicate
        let update = UpdateAgentSkill {
            name: Some(skill.name.clone()),
            display_title: Some(skill.name.clone()),
            description: Some(skill.description.clone()),
            skill_content: Some(skill.body.clone()),
            additional_files_json: None,
            enabled: Some(1),
            version: Some(existing.version + 1),
            updated_at: chrono::Utc::now().naive_utc(),
            source_path: source_path.clone(),
            category: skill.category.clone(),
            requires_sandbox: Some(if skill.requires_sandbox { 1 } else { 0 }),
            sandbox_config: sandbox_config_json.clone(),
            execution_mode: Some(skill.execution_mode.clone().unwrap_or_else(|| "direct".to_string())),
        };

        diesel::update(agent_skills::table.filter(agent_skills::id.eq(&existing.id)))
            .set(&update)
            .execute(&mut conn)
            .map_err(|e| format!("Failed to update skill: {}", e))?;

        // Delete old skill files before inserting new ones
        diesel::delete(schema::skill_files::table.filter(schema::skill_files::skill_id.eq(&existing.id)))
            .execute(&mut conn)
            .ok();

        existing.id
    } else {
        // Create new skill
        let new_id = uuid::Uuid::new_v4().to_string();
        let new_skill = NewAgentSkill {
            id: new_id.clone(),
            name: skill.name.clone(),
            display_title: skill.name.clone(),
            description: skill.description.clone(),
            skill_content: skill.body.clone(),
            additional_files_json: None,
            enabled: 1,
            version: 1,
            created_at: chrono::Utc::now().naive_utc(),
            updated_at: chrono::Utc::now().naive_utc(),
            source_path,
            category: skill.category.clone(),
            requires_sandbox: if skill.requires_sandbox { 1 } else { 0 },
            sandbox_config: sandbox_config_json,
            execution_mode: skill.execution_mode.clone().unwrap_or_else(|| "direct".to_string()),
        };

        diesel::insert_into(agent_skills::table)
            .values(&new_skill)
            .execute(&mut conn)
            .map_err(|e| format!("Failed to insert skill: {}", e))?;

        new_id
    };

    // Insert skill files
    for (relative_path, file_content) in loaded.files {
        let file = NewSkillFile {
            id: uuid::Uuid::new_v4().to_string(),
            skill_id: skill_id.clone(),
            relative_path,
            content: file_content.content,
            file_type: file_content.file_type,
            created_at: chrono::Utc::now().naive_utc(),
        };

        diesel::insert_into(schema::skill_files::table)
            .values(&file)
            .execute(&mut conn)
            .ok(); // Ignore errors for individual files
    }

    agent_skills::table
        .filter(agent_skills::id.eq(&skill_id))
        .first::<AgentSkill>(&mut conn)
        .map_err(|e| e.to_string())
}

// ==================== MARKETPLACE COMMANDS ====================

#[tauri::command]
pub async fn list_marketplace_skills(
    state: State<'_, AppState>,
) -> Result<Vec<MarketplaceSkill>, String> {
    // Skills directory is in src-tauri/skills
    let skills_dir = std::env::current_dir()
        .map_err(|e| e.to_string())?
        .join("src-tauri")
        .join("skills");

    // Fallback paths for different execution contexts
    let skills_dirs = [
        skills_dir.clone(),
        std::path::PathBuf::from("skills"),
        std::path::PathBuf::from("src-tauri/skills"),
    ];

    let mut marketplace_skills: Vec<MarketplaceSkill> = Vec::new();

    for dir in skills_dirs {
        if dir.is_dir() {
            if let Ok(skills) = scan_marketplace_skills(&dir) {
                // Get installed skill names
                let installed_names: Vec<String> = {
                    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;
                    schema::agent_skills::table
                        .select(schema::agent_skills::name)
                        .load::<String>(&mut conn)
                        .unwrap_or_default()
                };

                for skill_info in skills {
                    let is_installed = installed_names.contains(&skill_info.name);
                    marketplace_skills.push(MarketplaceSkill {
                        id: skill_info.name.clone(),
                        name: skill_info.name,
                        display_title: skill_info.display_title,
                        description: skill_info.description,
                        category: skill_info.category,
                        dir_name: skill_info.dir_name,
                        is_installed,
                    });
                }
                break;
            }
        }
    }

    Ok(marketplace_skills)
}

#[tauri::command]
pub async fn install_marketplace_skill(
    state: State<'_, AppState>,
    skill_dir_name: String,
) -> Result<AgentSkill, String> {
    // Find the skill directory
    let skills_dirs = [
        std::env::current_dir()
            .unwrap_or_default()
            .join("src-tauri")
            .join("skills"),
        std::path::PathBuf::from("skills"),
        std::path::PathBuf::from("src-tauri/skills"),
    ];

    for dir in skills_dirs {
        let skill_path = dir.join(&skill_dir_name);
        if skill_path.is_dir() && skill_path.join("SKILL.md").exists() {
            let loaded = load_skill_from_directory(&skill_path)?;
            return save_loaded_skill(&state, loaded, Some(skill_path.to_string_lossy().to_string()))
                .await;
        }
    }

    Err(format!("Skill '{}' not found in marketplace", skill_dir_name))
}

// ==================== SKILL FILES ====================

#[tauri::command]
pub async fn get_skill_files(
    state: State<'_, AppState>,
    skill_id: String,
) -> Result<Vec<SkillFile>, String> {
    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    schema::skill_files::table
        .filter(schema::skill_files::skill_id.eq(skill_id))
        .order(schema::skill_files::relative_path.asc())
        .load::<SkillFile>(&mut conn)
        .map_err(|e| e.to_string())
}

// ==================== AGENT SKILL ASSIGNMENTS ====================

#[tauri::command]
pub async fn assign_skill_to_agent(
    state: State<'_, AppState>,
    agent_id: String,
    skill_id: String,
) -> Result<(), String> {
    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    // Check if assignment already exists
    let exists: bool = schema::agent_skill_assignments::table
        .filter(schema::agent_skill_assignments::agent_id.eq(&agent_id))
        .filter(schema::agent_skill_assignments::skill_id.eq(&skill_id))
        .first::<AgentSkillAssignment>(&mut conn)
        .is_ok();

    if exists {
        return Ok(()); // Already assigned
    }

    let assignment = NewAgentSkillAssignment {
        agent_id,
        skill_id,
        created_at: chrono::Utc::now().naive_utc(),
    };

    diesel::insert_into(schema::agent_skill_assignments::table)
        .values(&assignment)
        .execute(&mut conn)
        .map_err(|e| format!("Failed to assign skill: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn unassign_skill_from_agent(
    state: State<'_, AppState>,
    agent_id: String,
    skill_id: String,
) -> Result<(), String> {
    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    diesel::delete(
        schema::agent_skill_assignments::table
            .filter(schema::agent_skill_assignments::agent_id.eq(agent_id))
            .filter(schema::agent_skill_assignments::skill_id.eq(skill_id)),
    )
    .execute(&mut conn)
    .map_err(|e| format!("Failed to unassign skill: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn get_agent_skills(
    state: State<'_, AppState>,
    agent_id: String,
) -> Result<Vec<AgentSkill>, String> {
    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    // Get skill IDs assigned to this agent
    let skill_ids: Vec<String> = schema::agent_skill_assignments::table
        .filter(schema::agent_skill_assignments::agent_id.eq(&agent_id))
        .select(schema::agent_skill_assignments::skill_id)
        .load(&mut conn)
        .map_err(|e| e.to_string())?;

    if skill_ids.is_empty() {
        return Ok(vec![]);
    }

    // Get the actual skills
    schema::agent_skills::table
        .filter(schema::agent_skills::id.eq_any(skill_ids))
        .order(schema::agent_skills::name.asc())
        .load::<AgentSkill>(&mut conn)
        .map_err(|e| e.to_string())
}

// ==================== DOCKER SANDBOX ====================

#[tauri::command]
pub async fn check_docker_available() -> Result<bool, String> {
    Ok(DockerSandbox::check_available().await)
}

// ==================== AGENT SCOPE ====================

#[tauri::command]
pub async fn update_agent_scope(
    state: State<'_, AppState>,
    agent_id: String,
    scope_type: String,
    workspace_path: Option<String>,
) -> Result<(), String> {
    use schema::agents::dsl::agents;

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    // Validate scope_type
    if scope_type != "global" && scope_type != "workspace" {
        return Err("Invalid scope_type. Must be 'global' or 'workspace'".to_string());
    }

    // If workspace scope, require workspace_path
    if scope_type == "workspace" && workspace_path.is_none() {
        return Err("workspace_path is required for 'workspace' scope".to_string());
    }

    diesel::update(agents.filter(schema::agents::id.eq(&agent_id)))
        .set((
            schema::agents::scope_type.eq(Some(scope_type)),
            schema::agents::workspace_path.eq(workspace_path),
        ))
        .execute(&mut conn)
        .map_err(|e| format!("Failed to update agent scope: {}", e))?;

    Ok(())
}

// ==================== CLEANUP ====================

/// Remove duplicate skills from the database, keeping only the most recently updated one
#[tauri::command]
pub async fn cleanup_duplicate_skills(state: State<'_, AppState>) -> Result<u32, String> {
    use schema::agent_skills;
    use std::collections::HashMap;

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    // Get all skills
    let all_skills: Vec<AgentSkill> = agent_skills::table
        .order(agent_skills::updated_at.desc())
        .load(&mut conn)
        .map_err(|e| e.to_string())?;

    // Group by name, keeping track of which to delete
    let mut seen_names: HashMap<String, String> = HashMap::new(); // name -> id to keep
    let mut ids_to_delete: Vec<String> = Vec::new();

    for skill in all_skills {
        if let Some(_existing_id) = seen_names.get(&skill.name) {
            // This is a duplicate (older one since we sorted by updated_at desc)
            ids_to_delete.push(skill.id);
        } else {
            // First occurrence (most recent), keep it
            seen_names.insert(skill.name, skill.id);
        }
    }

    let deleted_count = ids_to_delete.len() as u32;

    // Delete duplicates
    for id in ids_to_delete {
        // Delete skill files first
        diesel::delete(schema::skill_files::table.filter(schema::skill_files::skill_id.eq(&id)))
            .execute(&mut conn)
            .ok();

        // Delete skill assignments
        diesel::delete(
            schema::agent_skill_assignments::table
                .filter(schema::agent_skill_assignments::skill_id.eq(&id)),
        )
        .execute(&mut conn)
        .ok();

        // Delete the skill
        diesel::delete(agent_skills::table.filter(agent_skills::id.eq(&id)))
            .execute(&mut conn)
            .ok();
    }

    log::info!("Cleaned up {} duplicate skills", deleted_count);
    Ok(deleted_count)
}
