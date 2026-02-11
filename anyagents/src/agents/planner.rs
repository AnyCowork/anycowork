use crate::llm::LlmClient;
use crate::models::Plan;
use log::error;
use serde_json::Value;

pub struct PlanningAgent {
    pub model: String,
    pub provider: String,
    pub api_key: Option<String>,
}

impl PlanningAgent {
    pub fn new(model: String, provider: String, api_key: Option<String>) -> Self {
        Self { model, provider, api_key }
    }

    pub async fn plan<F>(&self, objective: &str, history: &str, on_token: F) -> Result<Plan, String>
    where
        F: Fn(String) + Send + Sync + 'static,
    {
        let schema = schemars::schema_for!(Plan);
        let schema_str = serde_json::to_string_pretty(&schema).map_err(|e| e.to_string())?;

        // Load Jinja2 template
        let template_str = std::fs::read_to_string("anyagents/prompts/planning_agent.j2")
            .unwrap_or_else(|_| include_str!("../../prompts/planning_agent.j2").to_string());

        let mut env = minijinja::Environment::new();
        env.add_template("planning", &template_str)
            .map_err(|e| e.to_string())?;

        let tmpl = env.get_template("planning").map_err(|e| e.to_string())?;
        let preamble = tmpl
            .render(minijinja::context! {
                schema => schema_str,
                scratchpad => Value::Null,
                context => history
            })
            .map_err(|e| e.to_string())?;

        let mut attempts = 0;
        let max_attempts = 3;

        loop {
            attempts += 1;

            let mut client = LlmClient::new(&self.provider, &self.model).with_preamble(&preamble);
            if let Some(key) = &self.api_key {
                client = client.with_api_key(key);
            }

            let result = client.stream_prompt(objective, &on_token).await;

            match result {
                Ok(full_response) if !full_response.is_empty() => {
                    // Clean up response if it contains markdown code blocks or preamble
                    let clean_json = clean_json_text(&full_response);

                    match serde_json::from_str::<Plan>(&clean_json) {
                        Ok(plan) => return Ok(plan),
                        Err(e) => {
                            error!(
                                "Failed to parse Plan JSON: {}. Attempt {}/{}",
                                e, attempts, max_attempts
                            );
                            if attempts >= max_attempts {
                                let snippet = if full_response.len() > 200 {
                                    format!("{}...", &full_response[..200])
                                } else {
                                    full_response.clone()
                                };
                                return Err(format!(
                                    "Failed to parse generated plan: {}. Response start: '{}'",
                                    e, snippet
                                ));
                            }
                        }
                    }
                }
                Ok(_) => {
                    error!("Planning attempt {} failed: Empty response", attempts);
                    if attempts >= max_attempts {
                        return Err(format!(
                            "Planning failed after {} attempts: Empty response",
                            max_attempts
                        ));
                    }
                }
                Err(e) => {
                    error!("Planning attempt {} failed: {}", attempts, e);
                    if attempts >= max_attempts {
                        return Err(format!(
                            "Planning failed after {} attempts: {}",
                            max_attempts, e
                        ));
                    }
                }
            }

            // Exponential backoff
            let wait_ms = 1000 * (2u64.pow(attempts as u32 - 1));
            on_token(format!(
                "\n⚠️ Attempt {} failed. Retrying in {}s...\n",
                attempts,
                wait_ms as f64 / 1000.0
            ));
            tokio::time::sleep(tokio::time::Duration::from_millis(wait_ms)).await;
        }
    }
}

fn clean_json_text(text: &str) -> String {
    // robustly find the JSON object frame
    let start = text.find('{');
    let end = text.rfind('}');

    match (start, end) {
        (Some(s), Some(e)) if s <= e => text[s..=e].to_string(),
        _ => text.trim().to_string(), // Fallback to original trim if no braces found
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_json_text_simple() {
        let json = r#"{"name": "test"}"#;
        assert_eq!(clean_json_text(json), json);
    }

    #[test]
    fn test_clean_json_text_with_markdown() {
        let text = r#"```json
{"name": "test", "value": 42}
```"#;
        assert_eq!(clean_json_text(text), r#"{"name": "test", "value": 42}"#);
    }

    #[test]
    fn test_clean_json_text_with_preamble() {
        let text = r#"Here is the plan:

{"tasks": [{"description": "Task 1"}]}

I hope this helps!"#;
        assert_eq!(
            clean_json_text(text),
            r#"{"tasks": [{"description": "Task 1"}]}"#
        );
    }

    #[test]
    fn test_clean_json_text_nested_braces() {
        let text = r#"Sure, here's the JSON:
{
  "objective": "Create a test",
  "tasks": [
    {
      "id": 1,
      "description": "Write test"
    }
  ]
}
Done!"#;
        let result = clean_json_text(text);
        assert!(result.starts_with('{'));
        assert!(result.ends_with('}'));
        assert!(result.contains("\"objective\""));
    }

    #[test]
    fn test_clean_json_text_no_braces() {
        let text = "No JSON here, just plain text";
        assert_eq!(clean_json_text(text), text);
    }

    #[test]
    fn test_clean_json_text_whitespace() {
        let text = "   { \"key\": \"value\" }   ";
        assert_eq!(clean_json_text(text), "{ \"key\": \"value\" }");
    }

    #[test]
    fn test_planning_agent_creation() {
        let agent = PlanningAgent::new("gpt-4".to_string(), "openai".to_string());
        assert_eq!(agent.model, "gpt-4");
        assert_eq!(agent.provider, "openai");
    }
}
