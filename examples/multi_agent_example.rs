/// Example: Multi-Agent Coordination System
/// 
/// This example demonstrates separate AgentCoordinator instances for
/// planner, executor, and reviewer agents working together.

use anycowork_core::prelude::*;
use rig::providers::openai;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// ============================================================================
// Data Structures
// ============================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Plan {
    tasks: Vec<TaskSpec>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct TaskSpec {
    id: String,
    description: String,
    dependencies: Vec<String>,
}

// ============================================================================
// Execution Tools (for Executor Agent)
// ============================================================================

#[derive(Debug, Deserialize)]
struct FileWriteArgs {
    path: String,
    content: String,
}

#[derive(Tool)]
#[tool(
    name = "write_file",
    description = "Write content to a file at the specified path"
)]
struct FileWriteTool {
    workspace: std::path::PathBuf,
}

impl FileWriteTool {
    async fn definition(&self, args: FileWriteArgs) -> Result<String, ToolError> {
        let full_path = self.workspace.join(&args.path);
        
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| ToolError::ExecutionError(e.to_string()))?;
        }
        
        std::fs::write(&full_path, args.content)
            .map_err(|e| ToolError::ExecutionError(e.to_string()))?;
        
        Ok(format!("Successfully wrote to {}", args.path))
    }
}

#[derive(Debug, Deserialize)]
struct FileReadArgs {
    path: String,
}

#[derive(Tool)]
#[tool(
    name = "read_file",
    description = "Read content from a file at the specified path"
)]
struct FileReadTool {
    workspace: std::path::PathBuf,
}

impl FileReadTool {
    async fn definition(&self, args: FileReadArgs) -> Result<String, ToolError> {
        let full_path = self.workspace.join(&args.path);
        
        std::fs::read_to_string(&full_path)
            .map_err(|e| ToolError::ExecutionError(e.to_string()))
    }
}

// ============================================================================
// Agent Creation Functions
// ============================================================================

/// Create a dedicated planner agent instance
async fn create_planner_agent(
    events: Arc<EventChannel>,
    session_id: String,
) -> Result<AgentCoordinator<openai::CompletionModel>, String> {
    let client = openai::Client::from_env();
    
    let agent = client
        .agent("gpt-4o")
        .preamble(
            r#"You are a planning agent that creates structured execution plans.

When given an objective, you must:
1. Break it down into discrete, actionable tasks
2. Identify dependencies between tasks
3. Return a JSON plan in this exact format:

{
  "tasks": [
    {
      "id": "task-1",
      "description": "Clear description of what to do",
      "dependencies": []
    },
    {
      "id": "task-2", 
      "description": "Another task",
      "dependencies": ["task-1"]
    }
  ]
}

Rules:
- Each task must have a unique ID (task-1, task-2, etc.)
- Dependencies must reference existing task IDs
- Tasks should be ordered logically
- Be specific and actionable in descriptions
- Return ONLY the JSON, no other text or explanation"#
        )
        .build();
    
    Ok(AgentCoordinator::new(
        agent,
        events,
        session_id,
        Arc::new(Mutex::new(None)),
    ))
}

/// Create an executor agent instance with tools
async fn create_executor_agent(
    events: Arc<EventChannel>,
    session_id: String,
    workspace: std::path::PathBuf,
) -> Result<AgentCoordinator<openai::CompletionModel>, String> {
    let client = openai::Client::from_env();
    
    let agent = client
        .agent("gpt-4o")
        .preamble(
            "You are an execution agent. Your role is to complete tasks using available tools. \
             You can read and write files, execute commands, and perform other actions. \
             Always confirm task completion with specific details about what you did."
        )
        .tool(FileWriteTool { workspace: workspace.clone() })
        .tool(FileReadTool { workspace: workspace.clone() })
        .build();
    
    Ok(AgentCoordinator::new(
        agent,
        events,
        session_id,
        Arc::new(Mutex::new(None)),
    ))
}

/// Create a reviewer agent instance
async fn create_reviewer_agent(
    events: Arc<EventChannel>,
    session_id: String,
) -> Result<AgentCoordinator<openai::CompletionModel>, String> {
    let client = openai::Client::from_env();
    
    let agent = client
        .agent("gpt-4o")
        .preamble(
            "You are a review agent. Your role is to verify that completed work meets \
             requirements and quality standards. Provide constructive feedback and identify \
             any issues that need to be addressed. \
             Respond with 'APPROVED' if the work is good, or provide specific feedback."
        )
        .build();
    
    Ok(AgentCoordinator::new(
        agent,
        events,
        session_id,
        Arc::new(Mutex::new(None)),
    ))
}

// ============================================================================
// Multi-Agent System
// ============================================================================

struct MultiAgentSystem {
    planner: AgentCoordinator<openai::CompletionModel>,
    executor: AgentCoordinator<openai::CompletionModel>,
    reviewer: AgentCoordinator<openai::CompletionModel>,
    events: Arc<EventChannel>,
}

impl MultiAgentSystem {
    async fn new(session_id: String, workspace: std::path::PathBuf) -> Result<Self, String> {
        // Create shared event channel
        let events = Arc::new(EventChannel::new());
        
        // Create three separate agent instances
        let planner = create_planner_agent(
            events.clone(),
            format!("{}-planner", session_id),
        ).await?;
        
        let executor = create_executor_agent(
            events.clone(),
            format!("{}-executor", session_id),
            workspace,
        ).await?;
        
        let reviewer = create_reviewer_agent(
            events.clone(),
            format!("{}-reviewer", session_id),
        ).await?;
        
        Ok(Self {
            planner,
            executor,
            reviewer,
            events,
        })
    }
    
    /// Execute an objective using coordinated multi-agent workflow
    async fn execute(&mut self, objective: &str) -> Result<String, String> {
        println!("🎯 Objective: {}\n", objective);
        
        // Phase 1: Planning - Use dedicated planner agent
        println!("📋 Phase 1: Planning (Planner Agent)");
        println!("─────────────────────────────────────");
        
        let plan_prompt = format!(
            "Create a detailed execution plan for this objective:\n\n{}\n\n\
             Return the plan as JSON following the specified format.",
            objective
        );
        
        let plan_response = self.planner.chat_stream(&plan_prompt).await?;
        
        // Extract plan from planner's response
        let plan = self.extract_plan(&plan_response)?;
        
        println!("\n✅ Plan created with {} tasks\n", plan.tasks.len());
        for task in &plan.tasks {
            println!("  • {} (deps: {:?})", task.description, task.dependencies);
        }
        
        // Phase 2: Execution - Use executor agent for each task
        println!("\n⚙️  Phase 2: Execution (Executor Agent)");
        println!("─────────────────────────────────────");
        
        let mut completed_tasks = Vec::new();
        let mut task_results = HashMap::new();
        
        for task in &plan.tasks {
            // Check dependencies
            let deps_met = task.dependencies.iter()
                .all(|dep| completed_tasks.contains(dep));
            
            if !deps_met {
                return Err(format!("Dependencies not met for task: {}", task.id));
            }
            
            println!("\n🔨 Executing: {} ({})", task.description, task.id);
            
            // Build context from previous tasks
            let context = self.build_task_context(&completed_tasks, &task_results);
            
            let task_prompt = format!(
                "Complete this task: {}\n\n\
                 Objective: {}\n\
                 Previous context:\n{}\n\n\
                 Use available tools to complete the task and report what you did.",
                task.description,
                objective,
                context
            );
            
            let result = self.executor.chat_stream(&task_prompt).await?;
            
            println!("✓ Task completed: {}", task.id);
            
            task_results.insert(task.id.clone(), result.clone());
            
            // Phase 3: Review - Use reviewer agent
            println!("🔍 Reviewing task: {} (Reviewer Agent)", task.id);
            
            let review_prompt = format!(
                "Review this completed task:\n\n\
                 Task: {}\n\
                 Result: {}\n\n\
                 Verify that:\n\
                 1. The task was completed as described\n\
                 2. The output meets quality standards\n\
                 3. No errors or issues are present\n\n\
                 Respond with 'APPROVED' if everything is good, or provide specific feedback.",
                task.description,
                result
            );
            
            let review = self.reviewer.chat_stream(&review_prompt).await?;
            
            if review.contains("APPROVED") {
                println!("✅ Review passed");
                completed_tasks.push(task.id.clone());
            } else {
                println!("⚠️  Review feedback: {}", review);
                
                // Re-execute with feedback using executor agent
                let retry_prompt = format!(
                    "The previous attempt needs revision. Feedback from reviewer:\n\n{}\n\n\
                     Please complete the task again addressing the feedback:\n{}",
                    review,
                    task.description
                );
                
                let retry_result = self.executor.chat_stream(&retry_prompt).await?;
                println!("✓ Task revised and completed: {}", task.id);
                
                task_results.insert(task.id.clone(), retry_result);
                completed_tasks.push(task.id.clone());
            }
        }
        
        println!("\n🎉 All tasks completed successfully!");
        
        Ok(format!("Completed {} tasks for objective: {}", completed_tasks.len(), objective))
    }
    
    /// Extract plan from planner agent's response
    fn extract_plan(&self, response: &str) -> Result<Plan, String> {
        // Find JSON in response
        if let Some(start) = response.find('{') {
            if let Some(end) = response.rfind('}') {
                let json_str = &response[start..=end];
                return serde_json::from_str(json_str)
                    .map_err(|e| format!("Failed to parse plan: {}", e));
            }
        }
        
        Err("No valid plan JSON found in planner response".to_string())
    }
    
    /// Build context from completed tasks
    fn build_task_context(
        &self,
        completed: &[String],
        results: &HashMap<String, String>,
    ) -> String {
        if completed.is_empty() {
            return "No previous tasks completed yet.".to_string();
        }
        
        completed.iter()
            .filter_map(|id| {
                results.get(id).map(|result| {
                    let truncated = if result.len() > 200 {
                        format!("{}...", &result[..200])
                    } else {
                        result.clone()
                    };
                    format!("  • {}: {}", id, truncated)
                })
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
    
    /// Subscribe to events for monitoring
    fn subscribe_events(&self) -> tokio::sync::broadcast::Receiver<AgentEvent> {
        self.events.subscribe()
    }
}

// ============================================================================
// Main Example
// ============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();
    
    // Create workspace
    let workspace = std::env::current_dir()?.join("workspace");
    std::fs::create_dir_all(&workspace)?;
    
    println!("🚀 Multi-Agent System Starting");
    println!("════════════════════════════════════════\n");
    
    // Create multi-agent system with separate instances
    let session_id = uuid::Uuid::new_v4().to_string();
    let mut system = MultiAgentSystem::new(session_id, workspace).await?;
    
    println!("✓ Created 3 agent instances:");
    println!("  • Planner Agent (creates execution plans)");
    println!("  • Executor Agent (completes tasks with tools)");
    println!("  • Reviewer Agent (validates quality)\n");
    
    // Subscribe to events for real-time monitoring
    let mut event_rx = system.subscribe_events();
    
    tokio::spawn(async move {
        while let Ok(event) = event_rx.recv().await {
            match event {
                AgentEvent::Token { content } => {
                    print!("{}", content);
                    std::io::Write::flush(&mut std::io::stdout()).ok();
                }
                AgentEvent::Thinking { message } => {
                    println!("\n💭 {}", message);
                }
                AgentEvent::StepStarted { step, .. } => {
                    println!("\n🔧 Tool: {} - {}", step.tool_name, step.tool_args);
                }
                AgentEvent::StepCompleted { step, .. } => {
                    if let Some(result) = step.result {
                        let truncated = if result.len() > 100 {
                            format!("{}...", &result[..100])
                        } else {
                            result
                        };
                        println!("✓ Result: {}", truncated);
                    }
                }
                AgentEvent::Error { message, error } => {
                    eprintln!("\n❌ Error: {} - {:?}", message, error);
                }
                _ => {}
            }
        }
    });
    
    // Execute objective
    let objective = "Create a simple hello world program in Python with proper documentation";
    
    match system.execute(objective).await {
        Ok(summary) => {
            println!("\n{}", summary);
            println!("\n════════════════════════════════════════");
            println!("✅ Multi-Agent Execution Complete");
            Ok(())
        }
        Err(e) => {
            eprintln!("\n❌ Execution failed: {}", e);
            Err(e.into())
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_plan_extraction() {
        let response = r#"
        Here's the plan:
        {
            "tasks": [
                {
                    "id": "task-1",
                    "description": "Test task",
                    "dependencies": []
                }
            ]
        }
        "#;
        
        let events = Arc::new(EventChannel::new());
        let system = MultiAgentSystem {
            planner: todo!(),
            executor: todo!(),
            reviewer: todo!(),
            events,
        };
        
        let plan = system.extract_plan(response).unwrap();
        assert_eq!(plan.tasks.len(), 1);
        assert_eq!(plan.tasks[0].id, "task-1");
    }
    
    #[test]
    fn test_task_context_building() {
        let events = Arc::new(EventChannel::new());
        let system = MultiAgentSystem {
            planner: todo!(),
            executor: todo!(),
            reviewer: todo!(),
            events,
        };
        
        let completed = vec!["task-1".to_string()];
        let mut results = HashMap::new();
        results.insert("task-1".to_string(), "Completed successfully".to_string());
        
        let context = system.build_task_context(&completed, &results);
        assert!(context.contains("task-1"));
        assert!(context.contains("Completed successfully"));
    }
    
    #[tokio::test]
    async fn test_event_emission() {
        let events = Arc::new(EventChannel::new());
        let mut rx = events.subscribe();
        
        events.emit(AgentEvent::Token {
            content: "test".to_string(),
        });
        
        let event = rx.recv().await.unwrap();
        match event {
            AgentEvent::Token { content } => assert_eq!(content, "test"),
            _ => panic!("Wrong event type"),
        }
    }
}
