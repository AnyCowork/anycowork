use futures::{SinkExt, StreamExt};
use serde_json::json;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use std::env;
use std::time::Duration;
use tokio::time::timeout;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    env_logger::init();

    let api_key = env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set");
    // Explicitly using the model from Python example
    let model = "gemini-2.5-flash-native-audio-preview-12-2025"; 

    // Using v1beta as per docs
    let ws_url = format!(
        "wss://generativelanguage.googleapis.com/ws/google.ai.generativelanguage.v1beta.GenerativeService.BidiGenerateContent?key={}",
        api_key
    );

    println!("Connecting to {}", ws_url);

    let (ws_stream, _) = connect_async(&ws_url).await?;
    println!("Connected to WebSocket");

    let (mut write, mut read) = ws_stream.split();

    // Setup Message using camelCase as per JS SDK/REST standards
    let setup_msg = json!({
        "setup": {
            "model": format!("models/{}", model),
            "generationConfig": {
                "responseModalities": ["AUDIO"],
                "speechConfig": {
                    "voiceConfig": {
                        "prebuiltVoiceConfig": {
                            "voiceName": "Aoede"
                        }
                    }
                }
            }
        }
    });

    println!("Sending setup message: {}", setup_msg);
    if let Err(e) = write.send(Message::Text(setup_msg.to_string())).await {
        eprintln!("Failed to send setup message: {}", e);
        return Ok(());
    }

    println!("Setup message sent. Waiting for response...");

    // Read loop with timeout
    loop {
        match timeout(Duration::from_secs(10), read.next()).await {
            Ok(Some(msg)) => {
                match msg {
                    Ok(Message::Text(text)) => {
                        println!("Received: {}", text);
                        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) {
                            if parsed.get("setupComplete").is_some() {
                                println!("Setup Complete! Handshake successful.");
                                break; 
                            }
                            if let Some(error) = parsed.get("error") {
                                eprintln!("Error received: {:?}", error);
                                break;
                            }
                        }
                    }
                    Ok(Message::Close(frame)) => {
                        println!("Connection closed: {:?}", frame);
                        break;
                    }
                    Err(e) => {
                        eprintln!("WebSocket Error: {}", e);
                        break;
                    }
                    _ => {
                        println!("Received other message: {:?}", msg);
                    }
                }
            }
            Ok(None) => {
                println!("Stream ended.");
                break;
            }
            Err(_) => {
                eprintln!("Timeout waiting for response from server.");
                break;
            }
        }
    }

    Ok(())
}
