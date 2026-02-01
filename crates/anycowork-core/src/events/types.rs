//! Event types for agent communication

use serde::{Deserialize, Serialize};

/// Events emitted by the agent system
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgentEvent {
    /// Token received during streaming
    Token { content: String },

    /// Job has started
    JobStarted { job: ExecutionJob },

    /// Job has completed
    JobCompleted { job: ExecutionJob, message: String },

    /// A tool execution step has started
    StepStarted { job: ExecutionJob, step: ToolStep },

    /// A tool execution step has completed
    StepCompleted { job: ExecutionJob, step: ToolStep },

    /// Approval is required for a tool execution
    ApprovalRequired { job: ExecutionJob, step: ToolStep },

    /// A step has been approved
    StepApproved { job: ExecutionJob, step: ToolStep },

    /// A step has been rejected
    StepRejected { job: ExecutionJob, step: ToolStep },

    /// Agent is thinking/processing
    Thinking { message: String },

    /// An error occurred
    Error { message: String, error: Option<String> },

    /// Plan update (for planning agents)
    PlanUpdate { plan: PlanUpdate },
}

/// Represents an execution job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionJob {
    pub id: String,
    pub session_id: String,
    pub status: String,
    pub query: String,
    pub steps: Vec<ToolStep>,
    pub current_step_index: usize,
    pub created_at: String,
}

impl ExecutionJob {
    pub fn new(id: String, session_id: String, query: String) -> Self {
        Self {
            id,
            session_id,
            status: "running".to_string(),
            query,
            steps: vec![],
            current_step_index: 0,
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub fn with_status(mut self, status: &str) -> Self {
        self.status = status.to_string();
        self
    }
}

/// Represents a tool execution step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolStep {
    pub id: String,
    pub tool_name: String,
    pub tool_args: serde_json::Value,
    pub status: StepStatus,
    pub result: Option<String>,
    pub requires_approval: bool,
    pub created_at: String,
}

impl ToolStep {
    pub fn new(id: String, tool_name: String, tool_args: serde_json::Value) -> Self {
        Self {
            id,
            tool_name,
            tool_args,
            status: StepStatus::Pending,
            result: None,
            requires_approval: false,
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub fn with_status(mut self, status: StepStatus) -> Self {
        self.status = status;
        self
    }

    pub fn with_result(mut self, result: String) -> Self {
        self.result = Some(result);
        self
    }

    pub fn with_approval_required(mut self, required: bool) -> Self {
        self.requires_approval = required;
        self
    }
}

/// Status of a tool execution step
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    Pending,
    Running,
    WaitingApproval,
    Completed,
    Failed,
    Skipped,
}

impl std::fmt::Display for StepStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StepStatus::Pending => write!(f, "pending"),
            StepStatus::Running => write!(f, "running"),
            StepStatus::WaitingApproval => write!(f, "waiting_approval"),
            StepStatus::Completed => write!(f, "completed"),
            StepStatus::Failed => write!(f, "failed"),
            StepStatus::Skipped => write!(f, "skipped"),
        }
    }
}

/// Plan update event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanUpdate {
    pub plan_id: String,
    pub tasks: Vec<TaskState>,
}

/// State of a task in a plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskState {
    pub task_id: String,
    pub description: String,
    pub status: String,
    pub result: Option<String>,
}
