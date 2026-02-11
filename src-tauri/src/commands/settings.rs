use crate::AppState;
use anyagents::models::{NewSetting, Setting, UpdateSetting};
use anyagents::schema::settings;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct AIConfig {
    pub provider: Option<String>,
    pub anthropic_api_key: Option<String>,
    pub anthropic_model: Option<String>,
    pub openai_api_key: Option<String>,
    pub openai_model: Option<String>,
    pub gemini_api_key: Option<String>,
    pub gemini_model: Option<String>,
    pub max_tokens: Option<i32>,
    pub temperature: Option<f32>,
}

/// Get AI configuration from settings
#[tauri::command]
pub async fn get_ai_config(state: State<'_, AppState>) -> Result<AIConfig, String> {
    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    // Fetch all settings
    let all_settings = settings::table
        .load::<Setting>(&mut conn)
        .map_err(|e| format!("Failed to load settings: {}", e))?;

    // Helper function to get value
    let get_value = |key: &str| -> Option<String> {
        all_settings
            .iter()
            .find(|s| s.key == key)
            .and_then(|s| s.value.clone())
            .filter(|v| !v.is_empty())
    };

    Ok(AIConfig {
        provider: get_value("ai_provider").or(Some("openai".to_string())),
        anthropic_api_key: get_value("anthropic_api_key"),
        anthropic_model: get_value("anthropic_model").or(Some("claude-opus-4-5-20251101".to_string())),
        openai_api_key: get_value("openai_api_key"),
        openai_model: get_value("openai_model").or(Some("gpt-4o".to_string())),
        gemini_api_key: get_value("gemini_api_key"),
        gemini_model: get_value("gemini_model").or(Some("gemini-2.0-flash-exp".to_string())),
        max_tokens: get_value("max_tokens").and_then(|v| v.parse().ok()).or(Some(4096)),
        temperature: get_value("temperature").and_then(|v| v.parse().ok()).or(Some(0.7)),
    })
}

/// Update AI configuration
#[tauri::command]
pub async fn update_ai_config(
    state: State<'_, AppState>,
    config: AIConfig,
) -> Result<(), String> {
    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;

    // Update each setting if present
    let mut settings_to_update = Vec::new();

    if let Some(v) = config.provider {
        settings_to_update.push(("ai_provider", v));
    }
    if let Some(v) = config.anthropic_api_key {
        settings_to_update.push(("anthropic_api_key", v));
    }
    if let Some(v) = config.anthropic_model {
        settings_to_update.push(("anthropic_model", v));
    }
    if let Some(v) = config.openai_api_key {
        settings_to_update.push(("openai_api_key", v));
    }
    if let Some(v) = config.openai_model {
        settings_to_update.push(("openai_model", v));
    }
    if let Some(v) = config.gemini_api_key {
        settings_to_update.push(("gemini_api_key", v));
    }
    if let Some(v) = config.gemini_model {
        settings_to_update.push(("gemini_model", v));
    }
    if let Some(v) = config.max_tokens {
        settings_to_update.push(("max_tokens", v.to_string()));
    }
    if let Some(v) = config.temperature {
        settings_to_update.push(("temperature", v.to_string()));
    }

    for (key, value) in settings_to_update {
        diesel::insert_into(settings::table)
            .values(NewSetting {
                id: uuid::Uuid::new_v4().to_string(), // New ID for insert
                key: key.to_string(),
                value: Some(value.clone()),
            })
            .on_conflict(settings::key)
            .do_update()
            .set(UpdateSetting {
                value: Some(value),
                updated_at: chrono::Utc::now().naive_utc(),
            })
            .execute(&mut conn)
            .map_err(|e| format!("Failed to update setting {}: {}", key, e))?;
    }

    log::info!("AI configuration updated successfully");
    Ok(())
}

/// Get available models for each provider
#[tauri::command]
pub async fn get_available_models() -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "providers": {
            "anthropic": {
                "models": [
                    {"id": "claude-opus-4-6-20250514", "name": "Claude Opus 4.6"},
                    {"id": "claude-opus-4-5-20251101", "name": "Claude Opus 4.5"},
                    {"id": "claude-sonnet-4-20250514", "name": "Claude Sonnet 4"},
                    {"id": "claude-sonnet-3-7-20250219", "name": "Claude Sonnet 3.7"},
                    {"id": "claude-haiku-3-5-20241022", "name": "Claude Haiku 3.5"}
                ]
            },
            "openai": {
                "models": [
                    {"id": "gpt-5", "name": "GPT-5"},
                    {"id": "gpt-4o", "name": "GPT-4o"},
                    {"id": "gpt-4-turbo", "name": "GPT-4 Turbo"},
                    {"id": "o1", "name": "O1"},
                    {"id": "o1-mini", "name": "O1 Mini"}
                ]
            },
            "gemini": {
                "models": [
                    {"id": "gemini-2.5-flash-native-audio-preview-12-2025", "name": "Gemini 2.5 Flash (Native Audio)"},
                    {"id": "gemini-2.5-pro-preview-12-2025", "name": "Gemini 2.5 Pro"},
                    {"id": "gemini-2.0-flash-exp", "name": "Gemini 2.0 Flash (Experimental)"},
                    {"id": "gemini-1.5-pro", "name": "Gemini 1.5 Pro"},
                    {"id": "gemini-1.5-flash", "name": "Gemini 1.5 Flash"}
                ]
            }
        }
    }))
}
