use crate::models::{NewTelegramConfig, TelegramConfig, UpdateTelegramConfig};
use crate::schema;
use crate::AppState;
use diesel::prelude::*;
use tauri::State;

#[tauri::command]
pub async fn create_telegram_config(
    state: State<'_, AppState>,
    bot_token: String,
    agent_id: String,
    allowed_chat_ids: Option<String>,
) -> Result<TelegramConfig, String> {
    use schema::telegram_configs;

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    let new_config = NewTelegramConfig {
        id: uuid::Uuid::new_v4().to_string(),
        bot_token,
        agent_id,
        is_active: 0,
        allowed_chat_ids,
        created_at: chrono::Utc::now().naive_utc(),
        updated_at: chrono::Utc::now().naive_utc(),
    };

    diesel::insert_into(telegram_configs::table)
        .values(&new_config)
        .execute(&mut conn)
        .map_err(|e| e.to_string())?;

    let created_id = new_config.id.clone();

    telegram_configs::table
        .filter(telegram_configs::id.eq(created_id))
        .first::<TelegramConfig>(&mut conn)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_telegram_configs(
    state: State<'_, AppState>,
) -> Result<Vec<TelegramConfig>, String> {
    use schema::telegram_configs::dsl::*;

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;
    telegram_configs
        .load::<TelegramConfig>(&mut conn)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_telegram_config(
    state: State<'_, AppState>,
    config_id: String,
) -> Result<TelegramConfig, String> {
    use schema::telegram_configs::dsl::*;

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;
    telegram_configs
        .filter(id.eq(config_id))
        .first::<TelegramConfig>(&mut conn)
        .map_err(|e| format!("Config not found: {}", e))
}

#[tauri::command]
pub async fn update_telegram_config(
    state: State<'_, AppState>,
    config_id: String,
    new_bot_token: Option<String>,
    new_agent_id: Option<String>,
    new_is_active: Option<i32>,
    new_allowed_chat_ids: Option<String>,
) -> Result<TelegramConfig, String> {
    use schema::telegram_configs::dsl::*;

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    let update = UpdateTelegramConfig {
        bot_token: new_bot_token,
        agent_id: new_agent_id,
        is_active: new_is_active,
        allowed_chat_ids: new_allowed_chat_ids,
        updated_at: chrono::Utc::now().naive_utc(),
    };

    diesel::update(telegram_configs.filter(id.eq(&config_id)))
        .set(&update)
        .execute(&mut conn)
        .map_err(|e| e.to_string())?;

    telegram_configs
        .filter(id.eq(&config_id))
        .first::<TelegramConfig>(&mut conn)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_telegram_config(
    state: State<'_, AppState>,
    config_id: String,
) -> Result<(), String> {
    use schema::telegram_configs::dsl::*;

    // Stop bot if running
    let _ = state.telegram_manager.stop_bot(&config_id).await;

    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;
    diesel::delete(telegram_configs.filter(id.eq(&config_id)))
        .execute(&mut conn)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn start_telegram_bot(
    state: State<'_, AppState>,
    config_id: String,
) -> Result<(), String> {
    state.telegram_manager.start_bot(&config_id).await
}

#[tauri::command]
pub async fn stop_telegram_bot(
    state: State<'_, AppState>,
    config_id: String,
) -> Result<(), String> {
    state.telegram_manager.stop_bot(&config_id).await
}

#[derive(serde::Serialize)]
pub struct TelegramBotStatus {
    pub config_id: String,
    pub is_running: bool,
}

#[tauri::command]
pub async fn get_telegram_bot_status(
    state: State<'_, AppState>,
    config_id: String,
) -> Result<TelegramBotStatus, String> {
    let is_running = state.telegram_manager.is_bot_running(&config_id).await;
    Ok(TelegramBotStatus {
        config_id,
        is_running,
    })
}

#[tauri::command]
pub async fn get_running_telegram_bots(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    Ok(state.telegram_manager.get_running_bot_ids().await)
}

// Response struct for test_telegram_bot
#[derive(serde::Serialize)]
pub struct TelegramBotTestResponse {
    pub success: bool,
    pub bot_username: Option<String>,
    pub error: Option<String>,
}

#[tauri::command]
pub async fn test_telegram_bot(bot_token: String) -> Result<TelegramBotTestResponse, String> {
    // Validate token format (should be in format: <bot_id>:<token_hash>)
    if !bot_token.contains(':') {
        return Ok(TelegramBotTestResponse {
            success: false,
            bot_username: None,
            error: Some("Invalid token format".to_string()),
        });
    }

    // Make request to Telegram API
    let url = format!("https://api.telegram.org/bot{}/getMe", bot_token);
    
    let client = reqwest::Client::new();
    match client.get(&url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                // Parse response
                match response.json::<serde_json::Value>().await {
                    Ok(json) => {
                        if let Some(result) = json.get("result") {
                            if let Some(username) = result.get("username").and_then(|u| u.as_str()) {
                                return Ok(TelegramBotTestResponse {
                                    success: true,
                                    bot_username: Some(username.to_string()),
                                    error: None,
                                });
                            }
                        }
                        Ok(TelegramBotTestResponse {
                            success: false,
                            bot_username: None,
                            error: Some("Unable to parse bot information".to_string()),
                        })
                    }
                    Err(e) => Ok(TelegramBotTestResponse {
                        success: false,
                        bot_username: None,
                        error: Some(format!("Failed to parse response: {}", e)),
                    }),
                }
            } else {
                Ok(TelegramBotTestResponse {
                    success: false,
                    bot_username: None,
                    error: Some(format!("Invalid bot token (HTTP {})", response.status())),
                })
            }
        }
        Err(e) => Ok(TelegramBotTestResponse {
            success: false,
            bot_username: None,
            error: Some(format!("Network error: {}", e)),
        }),
    }
}
