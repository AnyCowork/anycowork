//! Platform-agnostic event system for agent communication

mod channel;
mod types;

pub use channel::EventChannel;
pub use types::{AgentEvent, ExecutionJob, StepStatus, ToolStep};
