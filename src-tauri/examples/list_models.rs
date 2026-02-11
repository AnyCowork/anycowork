use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let api_key = env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set");
    let url = format!("https://generativelanguage.googleapis.com/v1beta/models?key={}", api_key);

    let client = reqwest::Client::new();
    let resp = client.get(&url).send().await?.text().await?;

    println!("Response: {}", resp);
    Ok(())
}
