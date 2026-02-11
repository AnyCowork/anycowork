// Voice calling service using Google Gemini Live API
// Implements real-time bidirectional audio streaming

use base64::{engine::general_purpose, Engine as _};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex, Notify};
use tokio_tungstenite::{connect_async, tungstenite::Message};

// Audio format constants
const INPUT_SAMPLE_RATE: u32 = 16000; // 16kHz input
const OUTPUT_SAMPLE_RATE: u32 = 24000; // 24kHz output
const BIT_DEPTH: u16 = 16; // 16-bit PCM
const CHANNELS: u8 = 1; // Mono

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceCallConfig {
    pub agent_id: String,
    pub agent_name: String,
    pub system_instruction: Option<String>,
    pub api_key: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum VoiceCallEvent {
    Connected,
    AudioData { data: Vec<u8> }, // PCM audio data from Gemini
    Transcription { text: String },
    Error { message: String },
    Disconnected,
}

pub struct VoiceCallSession {
    config: VoiceCallConfig,
    event_tx: mpsc::UnboundedSender<VoiceCallEvent>,
    audio_rx: Arc<Mutex<mpsc::UnboundedReceiver<Vec<u8>>>>,
    is_running: Arc<Mutex<bool>>,
    setup_complete: Arc<Notify>,
}

impl VoiceCallSession {
    pub fn new(
        config: VoiceCallConfig,
        event_tx: mpsc::UnboundedSender<VoiceCallEvent>,
        audio_rx: mpsc::UnboundedReceiver<Vec<u8>>,
    ) -> Self {
        Self {
            config,
            event_tx,
            audio_rx: Arc::new(Mutex::new(audio_rx)),
            is_running: Arc::new(Mutex::new(false)),
            setup_complete: Arc::new(Notify::new()),
        }
    }

    pub async fn start(&self) -> Result<(), String> {
        let mut running = self.is_running.lock().await;
        if *running {
            return Err("Session already running".to_string());
        }
        *running = true;
        drop(running);

        // Build WebSocket URL with API key
        let model = &self.config.model;
        let api_key = &self.config.api_key;
        let ws_url = format!(
            "wss://generativelanguage.googleapis.com/ws/google.ai.generativelanguage.v1beta.GenerativeService.BidiGenerateContent?key={}",
            api_key
        );

        log::info!("Connecting to Gemini Live API: {}", model);

        // Connect to WebSocket
        let (ws_stream, _) = connect_async(&ws_url)
            .await
            .map_err(|e| format!("WebSocket connection failed: {}", e))?;

        let (mut write, mut read) = ws_stream.split();

        // Send setup message
        let setup_msg = json!({
            "setup": {
                "model": format!("models/{}", model),
                "generation_config": {
                    "response_modalities": ["AUDIO"],
                    "speech_config": {
                        "voice_config": {
                            "prebuilt_voice_config": {
                                "voice_name": "Aoede" // Can be customized per agent
                            }
                        }
                    }
                }
            }
        });

        // Add system instruction if provided
        let setup_msg = if let Some(system_instruction) = &self.config.system_instruction {
            let mut msg = setup_msg;
            msg["setup"]["system_instruction"] = json!({
                "parts": [{
                    "text": format!("You are {}. {}", self.config.agent_name, system_instruction)
                }]
            });
            msg
        } else {
            setup_msg
        };

        write
            .send(Message::Text(setup_msg.to_string()))
            .await
            .map_err(|e| format!("Failed to send setup: {}", e))?;

        log::info!("Setup message sent, waiting for setupComplete...");

        // Clone for async tasks
        let event_tx = self.event_tx.clone();
        let audio_rx = self.audio_rx.clone();
        let is_running = self.is_running.clone();
        let setup_complete = self.setup_complete.clone();

        // Spawn task to handle incoming messages from Gemini
        let event_tx_clone = event_tx.clone();
        tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        log::info!("Received from Gemini: {}", text); // Log everything
                        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) {
                            // Check for error at top level
                            if let Some(error) = parsed.get("error") {
                                log::error!("Gemini API Error: {:?}", error);
                                let _ = event_tx_clone.send(VoiceCallEvent::Error {
                                    message: format!("Gemini Error: {}", error),
                                });
                            }

                            // Handle setup complete
                            if parsed.get("setupComplete").is_some() {
                                log::info!("Setup complete!");
                                let _ = event_tx_clone.send(VoiceCallEvent::Connected);
                                setup_complete.notify_one();
                            }

                            // Handle audio response
                            if let Some(server_content) = parsed.get("serverContent") {
                                if let Some(model_turn) = server_content.get("modelTurn") {
                                    if let Some(parts) = model_turn.get("parts") {
                                        if let Some(parts_array) = parts.as_array() {
                                            for part in parts_array {
                                                // Extract audio data
                                                if let Some(inline_data) = part.get("inlineData") {
                                                    if let Some(data_str) = inline_data.get("data").and_then(|d| d.as_str()) {
                                                        // Decode base64 audio
                                                        if let Ok(audio_bytes) = general_purpose::STANDARD.decode(data_str) {
                                                            let _ = event_tx_clone.send(VoiceCallEvent::AudioData {
                                                                data: audio_bytes,
                                                            });
                                                        }
                                                    }
                                                }

                                                // Extract text transcription
                                                if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                                                    let _ = event_tx_clone.send(VoiceCallEvent::Transcription {
                                                        text: text.to_string(),
                                                    });
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Ok(Message::Close(_)) => {
                        log::info!("WebSocket closed by server");
                        let _ = event_tx_clone.send(VoiceCallEvent::Disconnected);
                        setup_complete.notify_one(); // Unblock audio sender
                        break;
                    }
                    Err(e) => {
                        log::error!("WebSocket error: {}", e);
                        let _ = event_tx_clone.send(VoiceCallEvent::Error {
                            message: e.to_string(),
                        });
                        setup_complete.notify_one(); // Unblock audio sender
                        break;
                    }
                    _ => {}
                }
            }

            let mut running = is_running.lock().await;
            *running = false;
        });

        // Spawn task to send audio from microphone to Gemini
        let is_running = self.is_running.clone();
        let setup_complete = self.setup_complete.clone();
        tokio::spawn(async move {
            // Wait for setup to complete
            setup_complete.notified().await;

            let mut audio_rx_locked = audio_rx.lock().await;
            while let Some(audio_data) = audio_rx_locked.recv().await {
                // Check if session is still valid
                if !*is_running.lock().await {
                    break;
                }

                // Encode audio as base64
                let encoded = general_purpose::STANDARD.encode(&audio_data);

                // Create realtime input message
                let audio_msg = json!({
                    "realtimeInput": {
                        "mediaChunks": [{
                            "mimeType": format!("audio/pcm;rate={}", INPUT_SAMPLE_RATE),
                            "data": encoded
                        }]
                    }
                });

                if let Err(e) = write.send(Message::Text(audio_msg.to_string())).await {
                    // Ignore protocol errors caused by closing
                    if !e.to_string().contains("Sending after closing") {
                        log::error!("Failed to send audio: {}", e);
                         let _ = event_tx.send(VoiceCallEvent::Error {
                            message: format!("Failed to send audio: {}", e),
                        });
                    }
                    break;
                }
            }
            log::info!("Audio sender task finished");
        });

        Ok(())
    }

    pub async fn stop(&self) -> Result<(), String> {
        let mut running = self.is_running.lock().await;
        *running = false;
        Ok(())
    }

    pub async fn is_running(&self) -> bool {
        *self.is_running.lock().await
    }
}
