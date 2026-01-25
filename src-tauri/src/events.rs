use serde::Serialize;
use tauri::Emitter;

#[derive(Serialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgentEvent {
    // Basic Streaming
    Token {
        content: String,
    },

    // Job Lifecycle
    JobStarted {
        job: ExecutionJob,
    },
    JobCompleted {
        job: ExecutionJob,
        message: String,
    },

    // Step Lifecycle
    StepStarted {
        job: ExecutionJob,
        step: ExecutionStep,
    },
    StepCompleted {
        job: ExecutionJob,
        step: ExecutionStep,
    },

    // Approval
    ApprovalRequired {
        job: ExecutionJob,
        step: ExecutionStep,
    },
    StepApproved {
        job: ExecutionJob,
        step: ExecutionStep,
    },
    StepRejected {
        job: ExecutionJob,
        step: ExecutionStep,
    },

    // Status
    Thinking {
        message: String,
    },
    Error {
        message: String,
        error: Option<String>,
    },

    // Planning
    PlanUpdate {
        plan: crate::models::PlanUpdate,
    },
}

#[derive(Serialize, Clone, Debug)]
pub struct ExecutionJob {
    pub id: String,
    pub session_id: String,
    pub status: String, // running, waiting_approval, completed, failed
    pub query: String,
    pub steps: Vec<ExecutionStep>,
    pub current_step_index: usize,
    pub created_at: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct ExecutionStep {
    pub id: String,
    pub tool_name: String,
    pub tool_args: serde_json::Value,
    pub status: String,
    pub result: Option<String>,
    pub requires_approval: bool,
    pub created_at: String,
}

pub trait EventEmitter: Send + Sync {
    fn emit(
        &self,
        event: &str,
        payload: impl Serialize + Clone + Send + 'static,
    ) -> tauri::Result<()>;
}

impl EventEmitter for tauri::WebviewWindow {
    fn emit(
        &self,
        event: &str,
        payload: impl Serialize + Clone + Send + 'static,
    ) -> tauri::Result<()> {
        use tauri::Manager;
        self.app_handle().emit(event, payload)
    }
}
