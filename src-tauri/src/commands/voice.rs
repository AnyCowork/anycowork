use crate::voice_call::{VoiceCallConfig, VoiceCallEvent, VoiceCallSession};
use crate::AppState;
use anyagents::models::Agent;
use anyagents::schema::agents;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::sync::{mpsc, Mutex};

// Global map to track active voice call sessions
type SessionMap = Arc<Mutex<HashMap<String, Arc<VoiceCallSession>>>>;
type AudioSenderMap = Arc<Mutex<HashMap<String, mpsc::UnboundedSender<Vec<u8>>>>>;

#[derive(Clone, Serialize)]
struct VoiceEventPayload {
    session_id: String,
    event: VoiceCallEvent,
}

/// Start a voice call with an agent
#[tauri::command]
pub async fn start_voice_call(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    agent_id: String,
    api_key: String,
) -> Result<String, String> {
    log::info!("Starting voice call with agent: {}", agent_id);

    // Get agent details
    let mut conn = state.db_pool.get().map_err(|e| e.to_string())?;
    let agent = agents::table
        .filter(agents::id.eq(&agent_id))
        .first::<Agent>(&mut conn)
        .map_err(|e| format!("Agent not found: {}", e))?;

    // Generate session ID
    let session_id = uuid::Uuid::new_v4().to_string();

    // Create channels for communication
    let (event_tx, mut event_rx) = mpsc::unbounded_channel();
    let (audio_tx, audio_rx) = mpsc::unbounded_channel();

    // Create config
    let config = VoiceCallConfig {
        agent_id: agent.id.clone(),
        agent_name: agent.name.clone(),
        system_instruction: agent.system_prompt.clone(),
        api_key,
        model: "gemini-2.5-flash-native-audio-preview-12-2025".to_string(),
    };

    // Create session
    let session = Arc::new(VoiceCallSession::new(config, event_tx, audio_rx));

    // Store session in app state
    let sessions: SessionMap = app_handle.state::<SessionMap>().inner().clone();
    sessions
        .lock()
        .await
        .insert(session_id.clone(), session.clone());

    // Spawn task to forward events to frontend
    let session_id_clone = session_id.clone();
    let app_handle_clone = app_handle.clone();
    tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            let payload = VoiceEventPayload {
                session_id: session_id_clone.clone(),
                event,
            };

            if let Err(e) = app_handle_clone.emit("voice_call_event", &payload) {
                log::error!("Failed to emit voice event: {}", e);
            }
        }
    });

    // Start the session
    session.start().await?;

    // Store audio sender for this session
    let audio_senders: AudioSenderMap = app_handle.state::<AudioSenderMap>().inner().clone();
    audio_senders.lock().await.insert(session_id.clone(), audio_tx);

    log::info!("Voice call started with session ID: {}", session_id);
    Ok(session_id)
}

/// Stop an active voice call
#[tauri::command]
pub async fn stop_voice_call(app_handle: AppHandle, session_id: String) -> Result<(), String> {
    log::info!("Stopping voice call: {}", session_id);

    let sessions: SessionMap = app_handle.state::<SessionMap>().inner().clone();
    let mut sessions_locked = sessions.lock().await;

    if let Some(session) = sessions_locked.remove(&session_id) {
        session.stop().await?;
    }

    // Remove audio sender
    let audio_senders: AudioSenderMap = app_handle.state::<AudioSenderMap>().inner().clone();
    audio_senders.lock().await.remove(&session_id);

    log::info!("Voice call stopped: {}", session_id);
    Ok(())
}

/// Send audio data to an active voice call
#[tauri::command]
pub async fn send_voice_audio(
    app_handle: AppHandle,
    session_id: String,
    audio_data: Vec<u8>,
) -> Result<(), String> {
    let audio_senders: AudioSenderMap = app_handle.state::<AudioSenderMap>().inner().clone();
    let senders = audio_senders.lock().await;

    if let Some(sender) = senders.get(&session_id) {
        sender
            .send(audio_data)
            .map_err(|e| format!("Failed to send audio: {}", e))?;
        Ok(())
    } else {
        Err(format!("No active session found: {}", session_id))
    }
}

/// Check if a voice call is active
#[tauri::command]
pub async fn is_voice_call_active(
    app_handle: AppHandle,
    session_id: String,
) -> Result<bool, String> {
    let sessions: SessionMap = app_handle.state::<SessionMap>().inner().clone();
    let sessions_locked = sessions.lock().await;

    if let Some(session) = sessions_locked.get(&session_id) {
        Ok(session.is_running().await)
    } else {
        Ok(false)
    }
}

// Initialize voice call state
pub fn init_voice_state(app: &mut tauri::App) {
    let sessions: SessionMap = Arc::new(Mutex::new(HashMap::new()));
    let audio_senders: AudioSenderMap = Arc::new(Mutex::new(HashMap::new()));

    app.manage(sessions);
    app.manage(audio_senders);
}
