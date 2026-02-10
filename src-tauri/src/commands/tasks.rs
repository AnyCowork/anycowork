use tauri::{command, State};
use crate::AppState;
use crate::models::tasks::{Task, NewTask, UpdateTask};
use uuid::Uuid;
use chrono::Utc;
use diesel::prelude::*;

#[command]
pub async fn create_task(
    state: State<'_, AppState>,
    title_val: String,
    description_val: Option<String>,
    priority_val: Option<i32>,
    session_id_val: Option<String>,
    agent_id_val: Option<String>,
) -> Result<Task, String> {
    use crate::schema::tasks::dsl::*;

    let new_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    let new_task = NewTask {
        id: new_id.clone(),
        title: title_val,
        description: description_val,
        status: "pending".to_string(),
        priority: priority_val.unwrap_or(0),
        session_id: session_id_val,
        agent_id: agent_id_val,
        created_at: now.clone(),
        updated_at: now,
    };

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    diesel::insert_into(tasks)
        .values(&new_task)
        .execute(&mut conn)
        .map_err(|e| e.to_string())?;

    // Return the created task
    // We construct it manually to return it immediately without another query
    Ok(Task {
        id: new_task.id,
        title: new_task.title,
        description: new_task.description,
        status: new_task.status,
        priority: new_task.priority,
        session_id: new_task.session_id,
        agent_id: new_task.agent_id,
        created_at: new_task.created_at,
        updated_at: new_task.updated_at,
    })
}

#[command]
pub async fn list_tasks(
    state: State<'_, AppState>,
    session_id_filter: Option<String>,
    status_filter: Option<String>,
) -> Result<Vec<Task>, String> {
    use crate::schema::tasks::dsl::*;

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;
    
    let mut query = tasks.into_boxed();

    if let Some(sid) = session_id_filter {
        query = query.filter(session_id.eq(sid));
    }

    if let Some(s_status) = status_filter {
         query = query.filter(status.eq(s_status));
    }

    let results = query
        .order(created_at.desc())
        .load::<Task>(&mut conn)
        .map_err(|e| e.to_string())?;

    Ok(results)
}

#[command]
pub async fn update_task(
    state: State<'_, AppState>,
    task_id: String,
    data: UpdateTask,
) -> Result<Task, String> {
    use crate::schema::tasks::dsl::*;

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    // Add updated_at
    let mut update_data = data;
    update_data.updated_at = Utc::now().to_rfc3339();

    diesel::update(tasks.find(task_id.clone()))
        .set(&update_data)
        .execute(&mut conn)
        .map_err(|e| e.to_string())?;

    let updated_task = tasks
        .find(task_id)
        .first::<Task>(&mut conn)
        .map_err(|e| e.to_string())?;

    Ok(updated_task)
}

#[command]
pub async fn delete_task(
    state: State<'_, AppState>,
    task_id: String,
) -> Result<(), String> {
    use crate::schema::tasks::dsl::*;

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    diesel::delete(tasks.find(task_id))
        .execute(&mut conn)
        .map_err(|e| e.to_string())?;

    Ok(())
}
