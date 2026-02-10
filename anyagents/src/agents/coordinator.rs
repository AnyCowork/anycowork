use crate::agents::{planner::PlanningAgent, router::{Router, QueryType}, simple_chat::SimpleChatAgent, AgentLoop};
use crate::database::DbPool;
use crate::events::{AgentEvent, ExecutionJob, AgentObserver};
use crate::models::Agent as DbAgent;
use crate::permissions::PermissionManager;
use log::info;
use std::sync::Arc;
use tokio::sync::oneshot;
use uuid::Uuid;

pub struct Coordinator {
    pub session_id: String,
    pub agent_db: DbAgent,
    pub observer: Arc<dyn AgentObserver>,
    pub db_pool: DbPool,
    pub permission_manager: Arc<PermissionManager>,
    pub pending_approvals: Arc<dashmap::DashMap<String, oneshot::Sender<bool>>>,
    pub mode: String,
}

impl Coordinator {
    pub fn new(
        session_id: String,
        mut agent_db: DbAgent, // Make mutable to allow override
        observer: Arc<dyn AgentObserver>,
        db_pool: DbPool,
        permission_manager: Arc<PermissionManager>,
        pending_approvals: Arc<dashmap::DashMap<String, oneshot::Sender<bool>>>,
        mode: String,
        model_override: Option<String>,
    ) -> Self {
        // Apply model override if present
        if let Some(model) = model_override {
            agent_db.ai_model = model;
        }

        Self {
            session_id,
            agent_db,
            observer,
            db_pool,
            permission_manager,
            pending_approvals,
            mode,
        }
    }

    pub async fn run(&self, user_message: String) {
        let job_id = Uuid::new_v4().to_string();

        // Notify Job Started
        let job = ExecutionJob {
            id: job_id.clone(),
            session_id: self.session_id.clone(),
            status: "running".to_string(),
            query: user_message.clone(),
            steps: vec![],
            current_step_index: 0,
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        let _ = self.observer.emit(
            &format!("session:{}", self.session_id),
            serde_json::to_value(AgentEvent::JobStarted { job: job.clone() }).unwrap(),
        );

        // FAST MODE SHORT-CIRCUIT
        if self.mode == "fast" {
            let _ = self.observer.emit(
                &format!("session:{}", self.session_id),
                serde_json::to_value(AgentEvent::Thinking {
                    message: "Fast Mode: Executing directly...".to_string(),
                }).unwrap(),
            );

            let mut worker = AgentLoop::new(&self.agent_db, self.db_pool.clone()).await;
            worker.session_id = self.session_id.clone();

            worker
                .run(
                    user_message.clone(),
                    self.observer.clone(),
                    job_id.clone(),
                    self.pending_approvals.clone(),
                    self.permission_manager.clone(),
                    self.db_pool.clone(),
                )
                .await;

            return;
        }

        // SMART ROUTING: Classify query complexity
        let _ = self.observer.emit(
            &format!("session:{}", self.session_id),
            serde_json::to_value(AgentEvent::Thinking {
                message: "Analyzing query...".to_string(),
            }).unwrap(),
        );

        let router = Router::new(
            self.agent_db.ai_model.clone(),
            self.agent_db.ai_provider.clone(),
        );
        let query_type = router.classify(&user_message).await;
        info!("Query classified as: {:?}", query_type);

        // SIMPLE QUERY: Use simple chat agent (no tools)
        if query_type == QueryType::Simple {
            let _ = self.observer.emit(
                &format!("session:{}", self.session_id),
                serde_json::to_value(AgentEvent::Thinking {
                    message: "Responding...".to_string(),
                }).unwrap(),
            );

            let chat_agent = SimpleChatAgent::new(&self.agent_db);
            match chat_agent
                .chat(
                    &user_message,
                    &self.session_id,
                    self.observer.clone(),
                    &self.db_pool,
                )
                .await
            {
                Ok(response) => {
                    let _ = self.observer.emit(
                        &format!("session:{}", self.session_id),
                        serde_json::to_value(AgentEvent::JobCompleted {
                            job: ExecutionJob {
                                status: "completed".to_string(),
                                ..job.clone()
                            },
                            message: response,
                        }).unwrap(),
                    );
                }
                Err(e) => {
                    let _ = self.observer.emit(
                        &format!("session:{}", self.session_id),
                        serde_json::to_value(AgentEvent::Token {
                            content: format!("Error: {}", e),
                        }).unwrap_or(serde_json::Value::Null),
                    );
                    let _ = self.observer.emit(
                        &format!("session:{}", self.session_id),
                        serde_json::to_value(AgentEvent::JobCompleted {
                            job: ExecutionJob {
                                status: "failed".to_string(),
                                ..job.clone()
                            },
                            message: format!("Error: {}", e),
                        }).unwrap(),
                    );
                }
            }
            return;
        }

        // COMPLEX QUERY: Use planning-executor pattern
        // 1. Planning Phase
        let _ = self.observer.emit(
            &format!("session:{}", self.session_id),
            serde_json::to_value(AgentEvent::Thinking {
                message: "Analyzing request and creating a plan...".to_string(),
            }).unwrap(),
        );

        let planner = PlanningAgent::new(
            self.agent_db.ai_model.clone(),
            self.agent_db.ai_provider.clone(),
        );

        let observer_clone = self.observer.clone();
        let session_id_clone = self.session_id.clone();

        let on_token = move |token: String| {
            let _ = observer_clone.emit(
                &format!("session:{}", session_id_clone),
                serde_json::to_value(AgentEvent::Thinking { message: token }).unwrap_or(serde_json::Value::Null),
            );
        };

        let plan = match planner.plan(&user_message, on_token).await {
            Ok(p) => p,
            Err(e) => {
                let _ = self.observer.emit(
                    &format!("session:{}", self.session_id),
                    serde_json::to_value(AgentEvent::Token {
                        content: format!("Planning failed: {}", e),
                    }).unwrap_or(serde_json::Value::Null),
                );
                return;
            }
        };

        let mut plan_update = crate::models::PlanUpdate::from(plan.clone());
        let _ = self.observer.emit(
            &format!("session:{}", self.session_id),
            serde_json::to_value(AgentEvent::PlanUpdate {
                plan: plan_update.clone(),
            }).unwrap(),
        );

        // 2. Execution Phase
        // Initialize Worker (AgentLoop)
        // We reuse the same agent loop for sequential tasks to maintain context
        let mut worker = AgentLoop::new(&self.agent_db, self.db_pool.clone()).await;
        worker.session_id = self.session_id.clone();

        for (i, task) in plan.tasks.iter().enumerate() {
            // Update Task Status to Running
            plan_update.tasks[i].status = "running".to_string();
            let _ = self.observer.emit(
                &format!("session:{}", self.session_id),
                serde_json::to_value(AgentEvent::PlanUpdate {
                    plan: plan_update.clone(),
                }).unwrap(),
            );

            let _ = self.observer.emit(
                &format!("session:{}", self.session_id),
                serde_json::to_value(AgentEvent::Thinking {
                    message: format!("Starting Task: {}", task.description),
                }).unwrap(),
            );

            // Execute the valid task description
            worker
                .run(
                    task.description.clone(),
                    self.observer.clone(),
                    job_id.clone(),
                    self.pending_approvals.clone(),
                    self.permission_manager.clone(),
                    self.db_pool.clone(),
                )
                .await;

            // Update Task Status to Completed
            plan_update.tasks[i].status = "completed".to_string();
            let _ = self.observer.emit(
                &format!("session:{}", self.session_id),
                serde_json::to_value(AgentEvent::PlanUpdate {
                    plan: plan_update.clone(),
                }).unwrap(),
            );
        }

        // Finalize
        let _ = self.observer.emit(
            &format!("session:{}", self.session_id),
            serde_json::to_value(AgentEvent::JobCompleted {
                job: ExecutionJob {
                    status: "completed".to_string(),
                    ..job.clone()
                },
                message: "All tasks executed.".to_string(),
            }).unwrap(),
        );
    }
}
