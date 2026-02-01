use crate::events::{AgentEvent, EventChannel, ExecutionJob};
use futures::StreamExt;
use rig::agent::{Agent, MultiTurnStreamItem};
use rig::completion::{Chat, CompletionModel, Message};
use rig::one_or_many::OneOrMany;
use rig::completion::message::AssistantContent;
use rig::streaming::{StreamedAssistantContent, StreamingChat};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

pub struct AgentCoordinator<M: CompletionModel> {
    pub agent: Agent<M>,
    pub events: Arc<EventChannel>,
    pub session_id: String,
    pub chat_history: Vec<Message>,
    pub job_state: Arc<Mutex<Option<ExecutionJob>>>,
}

impl<M: CompletionModel> AgentCoordinator<M> 
where
    M: 'static,
    <M as CompletionModel>::StreamingResponse: Send,
{
    pub fn new(
        agent: Agent<M>, 
        events: Arc<EventChannel>, 
        session_id: String,
        job_state: Arc<Mutex<Option<ExecutionJob>>>
    ) -> Self {
        Self {
            agent,
            events,
            session_id,
            chat_history: Vec::new(),
            job_state,
        }
    }

    /// Non-streaming chat (legacy support)
    pub async fn chat(&mut self, prompt: &str) -> Result<String, String> {
        // Start Job
        let job_id = Uuid::new_v4().to_string();
        let job = ExecutionJob::new(job_id.clone(), self.session_id.clone(), prompt.to_string());
        
        self.events.emit(AgentEvent::JobStarted { job: job.clone() });

        // Update shared job state for the hook
        {
            let mut lock = self.job_state.lock().unwrap();
            *lock = Some(job.clone());
        }

        let history_clone = self.chat_history.clone();

        match self.agent.chat(prompt, history_clone).await {
            Ok(response) => {
                // Update History
                self.chat_history.push(Message::from(prompt.to_string()));
                
                let assistant_content = AssistantContent::from(response.clone());
                self.chat_history.push(Message::Assistant {
                    id: Some(Uuid::new_v4().to_string()),
                    content: OneOrMany::one(assistant_content),
                });

                // Complete Job
                let completed_job = ExecutionJob {
                    status: "completed".to_string(),
                    ..job
                };
                
                self.events.emit(AgentEvent::JobCompleted { 
                    job: completed_job,
                    message: response.clone() 
                });

                Ok(response)
            }
            Err(e) => {
                let error_msg = e.to_string();
                
                let failed_job = ExecutionJob {
                    status: "failed".to_string(),
                    ..job
                };
                
                self.events.emit(AgentEvent::JobCompleted { 
                    job: failed_job,
                    message: error_msg.clone()
                });
                
                Err(error_msg)
            }
        }
    }

    /// Streaming chat with real-time token emission
    /// This is the recommended method for interactive UIs
    pub async fn chat_stream(&mut self, prompt: &str) -> Result<String, String> {
        // Start Job
        let job_id = Uuid::new_v4().to_string();
        let job = ExecutionJob::new(job_id.clone(), self.session_id.clone(), prompt.to_string());
        
        self.events.emit(AgentEvent::JobStarted { job: job.clone() });

        // Update shared job state for the hook
        {
            let mut lock = self.job_state.lock().unwrap();
            *lock = Some(job.clone());
        }

        let history_clone = self.chat_history.clone();
        let mut full_response = String::new();

        // Create hook for event emission
        let hook = crate::agent::AnyCoworkHook::new(
            self.events.clone(),
            self.job_state.clone()
        );

        // Create streaming request with multi-turn support and hook
        let mut stream = self.agent
            .stream_chat(prompt, history_clone)
            .with_hook(hook) // Attach the hook!
            .multi_turn(10) // Allow up to 10 tool call rounds
            .await;

        // Process stream
        while let Some(item) = stream.next().await {
            match item {
                Ok(MultiTurnStreamItem::StreamAssistantItem(content)) => {
                    match content {
                        StreamedAssistantContent::Text(text) => {
                            // Emit token for real-time UI update
                            self.events.emit(AgentEvent::Token {
                                content: text.text.clone()
                            });
                            full_response.push_str(&text.text);
                        }
                        StreamedAssistantContent::ToolCall(tool_call) => {
                            // Tool call is handled by hook (on_tool_call)
                            // Emit thinking event to show what tool is being called
                            self.events.emit(AgentEvent::Thinking {
                                message: format!("Using tool: {}", tool_call.function.name)
                            });
                            log::debug!("Tool call: {}", tool_call.function.name);
                        }
                        StreamedAssistantContent::Reasoning(reasoning) => {
                            // Emit thinking event
                            let thinking_text = reasoning.reasoning.join("\n");
                            self.events.emit(AgentEvent::Thinking {
                                message: thinking_text
                            });
                        }
                        StreamedAssistantContent::ReasoningDelta { reasoning, .. } => {
                            // Emit partial thinking
                            self.events.emit(AgentEvent::Thinking {
                                message: reasoning
                            });
                        }
                        StreamedAssistantContent::ToolCallDelta { .. } => {
                            // Tool call delta - can be ignored or logged
                        }
                        StreamedAssistantContent::Final(_) => {
                            // Final response handled separately
                        }
                    }
                }
                Ok(MultiTurnStreamItem::StreamUserItem(_user_content)) => {
                    // Tool results - handled by hook (on_tool_result)
                }
                Ok(MultiTurnStreamItem::FinalResponse(final_res)) => {
                    // Final response with usage stats
                    log::info!("Final response usage: {:?}", final_res.usage());
                }
                Ok(_) => {
                    // Future variants - ignore for forward compatibility
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    log::error!("Stream error: {}", error_msg);
                    
                    let failed_job = ExecutionJob {
                        status: "failed".to_string(),
                        ..job
                    };
                    
                    self.events.emit(AgentEvent::JobCompleted { 
                        job: failed_job,
                        message: error_msg.clone()
                    });
                    
                    return Err(error_msg);
                }
            }
        }

        // Update History
        self.chat_history.push(Message::from(prompt.to_string()));
        
        let assistant_content = AssistantContent::from(full_response.clone());
        self.chat_history.push(Message::Assistant {
            id: Some(Uuid::new_v4().to_string()),
            content: OneOrMany::one(assistant_content),
        });

        // Complete Job
        let completed_job = ExecutionJob {
            status: "completed".to_string(),
            ..job
        };
        
        self.events.emit(AgentEvent::JobCompleted { 
            job: completed_job,
            message: full_response.clone() 
        });

        Ok(full_response)
    }
}
