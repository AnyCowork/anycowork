use crate::events::{AgentEvent, EventChannel, ExecutionJob, StepStatus, ToolStep};
use rig::agent::{CancelSignal, StreamingPromptHook};
use rig::completion::CompletionModel;
use std::future::Future;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct AnyCoworkHook {
    pub events: Arc<EventChannel>,
    pub job_state: Arc<Mutex<Option<ExecutionJob>>>,
}

impl AnyCoworkHook {
    pub fn new(events: Arc<EventChannel>, job_state: Arc<Mutex<Option<ExecutionJob>>>) -> Self {
        Self { events, job_state }
    }
}

impl<M: CompletionModel> StreamingPromptHook<M> for AnyCoworkHook {
    fn on_tool_call(
        &self,
        tool_name: &str,
        tool_call_id: Option<String>,
        args: &str,
        _cancel_sig: CancelSignal,
    ) -> impl Future<Output = ()> + Send {
        let job = self.job_state.lock().unwrap().clone();
        let events = self.events.clone();
        let tool_name = tool_name.to_string();
        
        async move {
            if let Some(job) = job {
                let step_id = tool_call_id.clone().unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
                
                let args_value: serde_json::Value = serde_json::from_str(args)
                    .unwrap_or(serde_json::Value::String(args.to_string()));
                
                let step = ToolStep::new(step_id, tool_name, args_value);

                events.emit(AgentEvent::StepStarted {
                    job,
                    step,
                });
            }
        }
    }

    fn on_tool_result(
        &self,
        tool_name: &str,
        tool_call_id: Option<String>,
        args: &str,
        result: &str,
        _cancel_sig: CancelSignal,
    ) -> impl Future<Output = ()> + Send {
        let job = self.job_state.lock().unwrap().clone();
        let events = self.events.clone();
        let tool_name = tool_name.to_string();
        let result = result.to_string();
        
        async move {
            if let Some(job) = job {
                let step_id = tool_call_id.clone().unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
                
                let args_value: serde_json::Value = serde_json::from_str(args)
                    .unwrap_or(serde_json::Value::String(args.to_string()));
                
                let step = ToolStep::new(step_id, tool_name, args_value)
                    .with_status(StepStatus::Completed)
                    .with_result(result);

                events.emit(AgentEvent::StepCompleted {
                    job,
                    step,
                });
            }
        }
    }
}
