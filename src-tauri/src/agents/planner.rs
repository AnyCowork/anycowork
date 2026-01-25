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

        // Stream handling
        // We will collect the full text to parse later
        let mut full_response = String::new();

        match self.provider.as_str() {
            "openai" => {
                let client = openai::Client::from_env();
                let agent = client.agent(&self.model).preamble(&preamble).build();
                let mut stream = agent.stream_prompt(objective).await;
                
                while let Some(chunk) = stream.next().await {
                    match chunk {
                        Ok(token) => {
                            if let MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::Text(t)) = token {
                                on_token(t.text.clone());
                                full_response.push_str(&t.text);
                            }
                        },
                        Err(e) => error!("Error in planning stream: {}", e),
                    }
                }
            },
            "gemini" => {
                let client = gemini::Client::from_env();
                let agent = client.agent(&self.model).preamble(&preamble).build();
                let mut stream = agent.stream_prompt(objective).await;

                while let Some(chunk) = stream.next().await {
                   match chunk {
                        Ok(token) => {
                            if let MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::Text(t)) = token {
                                on_token(t.text.clone());
                                full_response.push_str(&t.text);
                            }
                        },
                        Err(e) => error!("Error in planning stream: {}", e),
                    }
                }
            },
             "anthropic" => {
                 // This one was already taking 4 args? 
                 let client = anthropic::Client::from_env();
                let agent = client.agent(&self.model).preamble(&preamble).build();
                let mut stream = agent.stream_prompt(objective).await;
                
                while let Some(chunk) = stream.next().await {
                   match chunk {
                        Ok(token) => {
                            if let MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::Text(t)) = token {
                                on_token(t.text.clone());
                                full_response.push_str(&t.text);
                            }
                        },
                        Err(e) => error!("Error in planning stream: {}", e),
                    }
                }
            },
            _ => return Err(format!("Unsupported provider for planning: {}", self.provider)),
        };

        // Clean up response if it contains markdown code blocks or preamble
        let clean_json = clean_json_text(&full_response);

        match serde_json::from_str::<Plan>(&clean_json) {
            Ok(plan) => Ok(plan),
            Err(e) => {
                let snippet = if full_response.len() > 200 { 
                    format!("{}...", &full_response[..200]) 
                } else { 
                    full_response.clone() 
                };
                error!("Failed to parse Plan JSON: {}. Full Response: {}", e, full_response);
                Err(format!("Failed to parse generated plan: {}. Response start: '{}'", e, snippet))
            }
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
