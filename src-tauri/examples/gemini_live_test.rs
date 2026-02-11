use futures::{SinkExt, StreamExt};
use serde_json::json;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    env_logger::init();

    let api_key = env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set");
    let model = "gemini-2.5-flash-native-audio-preview-09-2025"; 

    let ws_url = format!(
        "wss://generativelanguage.googleapis.com/ws/google.ai.generativelanguage.v1beta.GenerativeService.BidiGenerateContent?key={}",
        api_key
    );

    println!("Connecting to {}", ws_url);

    let (ws_stream, _) = connect_async(&ws_url).await?;
    println!("Connected to WebSocket");

    let (mut write, mut read) = ws_stream.split();

    // Send Setup Message
    let setup_msg = json!({
        "setup": {
            "model": format!("models/{}", model),
            "generation_config": {
                "response_modalities": ["AUDIO"]
            }
        }
    });

    println!("Sending setup message: {}", setup_msg);
    write.send(Message::Text(setup_msg.to_string())).await?;

    // Read loop
    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                println!("Received: {}", text);
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) {
                    if parsed.get("setupComplete").is_some() {
                        println!("Setup Complete! Ready to stream audio.");
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
                eprintln!("Error: {}", e);
                break;
            }
            _ => {}
        }
    }

    Ok(())
}
