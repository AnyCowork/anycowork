use rig::providers::{openai, anthropic, gemini};
use crate::models::Plan;
use log::error;
use serde_json::Value;
use futures::StreamExt;
use rig::streaming::StreamingPrompt;
use rig::agent::MultiTurnStreamItem;
use rig::streaming::StreamedAssistantContent;
use rig::client::ProviderClient;
use rig::client::CompletionClient;

pub struct PlanningAgent {
    pub model: String,
    pub provider: String,
}

impl PlanningAgent {
    pub fn new(model: String, provider: String) -> Self {
        Self { model, provider }
    }

    pub async fn plan<F>(&self, objective: &str, on_token: F) -> Result<Plan, String> 
    where F: Fn(String) + Send + Sync + 'static
    {
        let schema = schemars::schema_for!(Plan);
        let schema_str = serde_json::to_string_pretty(&schema).map_err(|e| e.to_string())?;

        // Load Jinja2 template
        let template_str = std::fs::read_to_string("src-tauri/prompts/planning_agent.j2")
            .unwrap_or_else(|_| include_str!("../../prompts/planning_agent.j2").to_string());
        
        let mut env = minijinja::Environment::new();
        env.add_template("planning", &template_str).map_err(|e| e.to_string())?;
        
        let tmpl = env.get_template("planning").map_err(|e| e.to_string())?;
        let preamble = tmpl.render(minijinja::context! {
            schema => schema_str,
            scratchpad => Value::Null
        }).map_err(|e| e.to_string())?;

        let mut attempts = 0;
        let max_attempts = 3;

        loop {
            attempts += 1;
            
            // Stream handling
            // We will collect the full text to parse later
            let mut full_response = String::new();

            let result = match self.provider.as_str() {
                "openai" => {
                    let client = openai::Client::from_env();
                    let agent = client.agent(&self.model).preamble(&preamble).build();
                    let mut stream = agent.stream_prompt(objective).await;
                    
                    let mut res = Ok(());
                    
                    while let Some(chunk) = stream.next().await {
                        match chunk {
                            Ok(token) => {
                                if let MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::Text(t)) = token {
                                    on_token(t.text.clone());
                                    full_response.push_str(&t.text);
                                }
                            },
                            Err(e) => {
                                error!("Error in planning stream: {}", e);
                                res = Err(e.to_string());
                                break;
                            }
                        }
                    }
                    res
                },
                "gemini" => {
                    let client = gemini::Client::from_env();
                    let agent = client.agent(&self.model).preamble(&preamble).build();
                    let mut stream = agent.stream_prompt(objective).await;
                    let mut result = Ok(());

                    while let Some(chunk) = stream.next().await {
                    match chunk {
                            Ok(token) => {
                                if let MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::Text(t)) = token {
                                    on_token(t.text.clone());
                                    full_response.push_str(&t.text);
                                }
                            },
                            Err(e) => {
                                error!("Error in planning stream: {}", e);
                                result = Err(e.to_string());
                                break;
                            }
                        }
                    }
                    result
                },
                "anthropic" => {
                    let client = anthropic::Client::from_env();
                    let agent = client.agent(&self.model).preamble(&preamble).build();
                    let mut stream = agent.stream_prompt(objective).await;
                    let mut result = Ok(());

                    while let Some(chunk) = stream.next().await {
                    match chunk {
                            Ok(token) => {
                                if let MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::Text(t)) = token {
                                    on_token(t.text.clone());
                                    full_response.push_str(&t.text);
                                }
                            },
                            Err(e) => {
                                error!("Error in planning stream: {}", e);
                                result = Err(e.to_string());
                                break;
                            }
                        }
                    }
                    result
                },
                _ => return Err(format!("Unsupported provider for planning: {}", self.provider)),
            };

            // If we got a stream error or empty response, we retry
            if result.is_ok() && !full_response.is_empty() {
                // Clean up response if it contains markdown code blocks or preamble
                let clean_json = clean_json_text(&full_response);

                match serde_json::from_str::<Plan>(&clean_json) {
                    Ok(plan) => return Ok(plan),
                    Err(e) => {
                        error!("Failed to parse Plan JSON: {}. Attempt {}/{}", e, attempts, max_attempts);
                        if attempts >= max_attempts {
                            let snippet = if full_response.len() > 200 { 
                                format!("{}...", &full_response[..200]) 
                            } else { 
                                full_response.clone() 
                            };
                            return Err(format!("Failed to parse generated plan: {}. Response start: '{}'", e, snippet));
                        }
                    }
                }
            } else {
                let err_msg = result.err().unwrap_or_else(|| "Empty response".to_string());
                error!("Planning attempt {} failed: {}", attempts, err_msg);
                if attempts >= max_attempts {
                    return Err(format!("Planning failed after {} attempts: {}", max_attempts, err_msg));
                }
            }

            // Exponential backoff
            let wait_ms = 1000 * (2u64.pow(attempts as u32 - 1));
            on_token(format!("\n⚠️ Attempt {} failed. Retrying in {}s...\n", attempts, wait_ms as f64 / 1000.0));
            tokio::time::sleep(tokio::time::Duration::from_millis(wait_ms)).await;
        }
    }
}

fn clean_json_text(text: &str) -> String {
    // robustly find the JSON object frame
    let start = text.find('{');
    let end = text.rfind('}');
    
    match (start, end) {
        (Some(s), Some(e)) if s <= e => {
            text[s..=e].to_string()
        },
        _ => text.trim().to_string() // Fallback to original trim if no braces found
    }
}
