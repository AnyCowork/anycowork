use serde::{Deserialize, Serialize};
use crate::models::Plan;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlanUpdate {
    pub tasks: Vec<TaskState>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaskState {
    pub id: String,
    pub description: String,
    pub status: String, // "pending", "running", "completed", "failed"
    pub result: Option<String>,
}

impl From<Plan> for PlanUpdate {
    fn from(plan: Plan) -> Self {
        Self {
            tasks: plan.tasks.into_iter().map(|t| TaskState {
                id: t.id,
                description: t.description,
                status: "pending".to_string(),
                result: None,
            }).collect(),
        }
    }
}
