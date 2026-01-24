use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct Plan {
    pub tasks: Vec<TaskSpec>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct TaskSpec {
    pub id: String,
    pub description: String,
    /// List of Task IDs that must be completed before this task can start
    pub dependencies: Vec<String>,
}
