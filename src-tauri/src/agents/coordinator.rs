use crate::agents::{planner::PlanningAgent, AgentLoop};
use crate::database::DbPool;
use crate::events::{AgentEvent, ExecutionJob};
use crate::models::Agent as DbAgent;
use crate::permissions::PermissionManager;
use std::sync::Arc;
use tauri::{Emitter, Runtime};
use tokio::sync::oneshot;
use uuid::Uuid;

pub struct Coordinator<R: Runtime> {
    pub session_id: String,
    pub agent_db: DbAgent,
    pub window: tauri::WebviewWindow<R>,
    pub db_pool: DbPool,
    pub permission_manager: Arc<PermissionManager>,
    pub pending_approvals: Arc<dashmap::DashMap<String, oneshot::Sender<bool>>>,
    pub mode: String,
}

impl<R: Runtime> Coordinator<R> {
    pub fn new(
        session_id: String,
        agent_db: DbAgent,
        window: tauri::WebviewWindow<R>,
        db_pool: DbPool,
        permission_manager: Arc<PermissionManager>,
        pending_approvals: Arc<dashmap::DashMap<String, oneshot::Sender<bool>>>,
        mode: String,
    ) -> Self {
        Self {
            session_id,
            agent_db,
            window,
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
        let _ = Emitter::emit(
            &self.window,
            &format!("session:{}", self.session_id),
            AgentEvent::JobStarted { job: job.clone() },
        );

        // FAST MODE SHORT-CIRCUIT
        if self.mode == "fast" {
            let _ = Emitter::emit(
                &self.window,
                &format!("session:{}", self.session_id),
                AgentEvent::Thinking {
                    message: "Fast Mode: Executing directly...".to_string(),
                },
            );

            let mut worker = AgentLoop::new(&self.agent_db).await;
            worker.session_id = self.session_id.clone();

            worker
                .run(
                    user_message.clone(),
                    self.window.clone(),
                    job_id.clone(),
                    self.pending_approvals.clone(),
                    self.permission_manager.clone(),
                    self.db_pool.clone(),
                )
                .await;

            return;
        }

        // 1. Planning Phase
        let _ = Emitter::emit(
            &self.window,
            &format!("session:{}", self.session_id),
            AgentEvent::Thinking {
                message: "Analyzing request and creating a plan...".to_string(),
            },
        );

        let planner = PlanningAgent::new(
            self.agent_db.ai_model.clone(),
            self.agent_db.ai_provider.clone(),
        );

        let window_clone = self.window.clone();
        let session_id_clone = self.session_id.clone();

        let on_token = move |token: String| {
            let _ = Emitter::emit(
                &window_clone,
                &format!("session:{}", session_id_clone),
                AgentEvent::Thinking { message: token },
            );
        };

        let plan = match planner.plan(&user_message, on_token).await {
            Ok(p) => p,
            Err(e) => {
                let _ = Emitter::emit(
                    &self.window,
                    &format!("session:{}", self.session_id),
                    AgentEvent::Token {
                        content: format!("Planning failed: {}", e),
                    },
                );
                return;
            }
        };

        let mut plan_update = crate::models::PlanUpdate::from(plan.clone());
        let _ = Emitter::emit(
            &self.window,
            &format!("session:{}", self.session_id),
            AgentEvent::PlanUpdate {
                plan: plan_update.clone(),
            },
        );

        // 2. Execution Phase
        // Initialize Worker (AgentLoop)
        // We reuse the same agent loop for sequential tasks to maintain context
        let mut worker = AgentLoop::new(&self.agent_db).await;
        worker.session_id = self.session_id.clone();

        for (i, task) in plan.tasks.iter().enumerate() {
            // Update Task Status to Running
            plan_update.tasks[i].status = "running".to_string();
            let _ = Emitter::emit(
                &self.window,
                &format!("session:{}", self.session_id),
                AgentEvent::PlanUpdate {
                    plan: plan_update.clone(),
                },
            );

            let _ = Emitter::emit(
                &self.window,
                &format!("session:{}", self.session_id),
                AgentEvent::Thinking {
                    message: format!("Starting Task: {}", task.description),
                },
            );

            // Execute the valid task description
            // Note: We need to capture the result/output of the worker run if possible.
            // Currently AgentLoop::run is void. We might need to refactor AgentLoop::run to return result
            // OR listen to side effects. For now, let's assume success if it returns.
            worker
                .run(
                    task.description.clone(),
                    self.window.clone(),
                    job_id.clone(),
                    self.pending_approvals.clone(),
                    self.permission_manager.clone(),
                    self.db_pool.clone(),
                )
                .await;

            // Update Task Status to Completed
            plan_update.tasks[i].status = "completed".to_string();
            let _ = Emitter::emit(
                &self.window,
                &format!("session:{}", self.session_id),
                AgentEvent::PlanUpdate {
                    plan: plan_update.clone(),
                },
            );
        }

        // Finalize
        let _ = Emitter::emit(
            &self.window,
            &format!("session:{}", self.session_id),
            AgentEvent::JobCompleted {
                job: ExecutionJob {
                    status: "completed".to_string(),
                    ..job.clone()
                },
                message: "All tasks executed.".to_string(),
            },
        );
    }
}
