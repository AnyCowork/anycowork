use super::{Tool, ToolContext};
use crate::database::DbPool;
use crate::models::Agent;
use async_trait::async_trait;
use diesel::prelude::*;
use serde_json::{json, Value};

/// Tool to list all available colleagues/agents in the system
pub struct ListColleaguesTool {
    db_pool: DbPool,
    current_agent_id: String,
}

impl ListColleaguesTool {
    pub fn new(db_pool: DbPool, current_agent_id: String) -> Self {
        Self {
            db_pool,
            current_agent_id,
        }
    }
}

#[async_trait]
impl Tool for ListColleaguesTool {
    fn name(&self) -> &str {
        "list_colleagues"
    }

    fn description(&self) -> &str {
        "List all available colleagues/team members you can send emails to. Use this to find who you can communicate with internally. Returns each person's name and virtual email address."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {},
            "required": []
        })
    }

    async fn execute(&self, _args: Value, _ctx: &ToolContext) -> Result<Value, String> {
        let mut conn = self.db_pool.get().map_err(|e| format!("DB error: {}", e))?;

        use crate::schema::agents::dsl::*;
        let all_agents: Vec<Agent> = agents
            .filter(status.eq("active"))
            .load::<Agent>(&mut conn)
            .map_err(|e| format!("DB error: {}", e))?;

        let mut colleagues = Vec::new();

        for agent in all_agents {
            // Skip self
            if agent.id == self.current_agent_id {
                continue;
            }

            // Generate virtual email address
            let email = generate_virtual_email(&agent.name);

            colleagues.push(json!({
                "name": agent.name,
                "email": email,
                "avatar": agent.avatar.unwrap_or_else(|| "👤".to_string()),
                "description": agent.description.unwrap_or_else(|| "Team member".to_string()),
                "role": extract_role(&agent.name)
            }));
        }

        // Add user to contacts
        colleagues.push(json!({
            "name": "User",
            "email": "user@anycowork.local",
            "avatar": "👤",
            "description": "The user who owns this workspace",
            "role": "Owner"
        }));

        Ok(json!({
            "colleagues": colleagues,
            "count": colleagues.len(),
            "note": "You can send internal emails to any of these colleagues using just their name (e.g., 'Jordan' or 'Jordan the PM'). No external email address needed."
        }))
    }

    fn verify_result(&self, result: &Value) -> bool {
        result.get("colleagues").is_some()
    }
}

/// Generate a virtual email address for an agent
/// This is just for display/reference - the system uses names internally
fn generate_virtual_email(name: &str) -> String {
    // Convert "Jordan the PM" -> "jordan.pm@anycowork.local"
    let email_part = name
        .to_lowercase()
        .replace(" the ", ".")
        .replace(" ", ".")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '.')
        .collect::<String>();

    format!("{}@anycowork.local", email_part)
}

/// Extract role from name (e.g., "Jordan the PM" -> "PM")
fn extract_role(name: &str) -> String {
    if let Some(pos) = name.find(" the ") {
        name[pos + 5..].to_string()
    } else {
        "Team Member".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_virtual_email() {
        assert_eq!(
            generate_virtual_email("Jordan the PM"),
            "jordan.pm@anycowork.local"
        );
        assert_eq!(
            generate_virtual_email("Alex the Chief"),
            "alex.chief@anycowork.local"
        );
        assert_eq!(
            generate_virtual_email("Dev the Developer"),
            "dev.developer@anycowork.local"
        );
    }

    #[test]
    fn test_extract_role() {
        assert_eq!(extract_role("Jordan the PM"), "PM");
        assert_eq!(extract_role("Alex the Chief"), "Chief");
        assert_eq!(extract_role("Simple Name"), "Team Member");
    }
}
