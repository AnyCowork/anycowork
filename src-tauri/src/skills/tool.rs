use crate::models::SandboxConfig;
use crate::skills::docker::DockerSandbox;
use crate::skills::loader::LoadedSkill;
use crate::tools::{Tool, ToolContext};
use async_trait::async_trait;
use serde_json::{json, Value};

use tauri::Runtime;

pub struct SkillTool {
    pub name: String,
    pub description: String,
    pub skill: LoadedSkill,
    pub workspace_path: std::path::PathBuf,
    pub agent_execution_mode: String,
}

impl SkillTool {
    pub fn new(skill: LoadedSkill, workspace_path: std::path::PathBuf, agent_execution_mode: String) -> Self {
        // Build an enhanced description that instructs the LLM to read the skill content first
        let enhanced_description = format!(
            "{}. IMPORTANT: Before using this skill, call it with args='read' to get detailed instructions and code examples.",
            skill.skill.description.trim_end_matches('.')
        );

        Self {
            name: skill.skill.name.clone(),
            description: enhanced_description,
            skill,
            workspace_path,
            agent_execution_mode,
        }
    }
}

#[async_trait]
impl<R: Runtime> Tool<R> for SkillTool {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "args": {
                    "type": "string",
                    "description": "Either 'read' to get the full skill guide with code examples, or a shell command to execute. ALWAYS use 'read' first to learn how to use this skill properly."
                }
            },
            "required": ["args"]
        })
    }

    async fn execute(&self, args: Value, _ctx: &ToolContext<R>) -> Result<Value, String> {
        let command_str = args.get("args").and_then(|v| v.as_str()).unwrap_or("");
        
        // Handle "read" content request for Knowledge skills
        if command_str.trim().eq_ignore_ascii_case("read") {
             return Ok(json!({ "content": self.skill.skill.body }));
        }

        let mut sandbox = DockerSandbox::new();
        sandbox.init().await;

        // Check Docker availability first
        let docker_available = sandbox.is_available();
        
        // Determine final mode based on Agent preference AND Skill requirement
        // Agent preference takes precedence for safety (e.g. if agent=sandbox, we MUST use sandbox)
        let use_docker = match self.agent_execution_mode.as_str() {
             "sandbox" => {
                 if !docker_available {
                     return Err("Security Policy Enforcement: Sandbox mode is enabled but Docker is not available.".to_string());
                 }
                 true
             },
             "direct" => {
                 // Even if agent says direct, if skill REQUIRES sandbox, we must fail or error?
                 // Or we trust the user/agent is doing something unsafe intentionally?
                 // Let's protect: If skill requires sandbox, we fail in direct mode unless we can't.
                 if self.skill.skill.requires_sandbox {
                      return Err("Skill requires sandbox but Agent is in 'direct' execution mode.".to_string());
                 }
                 false
             },
             "flexible" | _ => {
                 // Fallback to skill preference
                 let skill_mode = self.skill.skill.execution_mode.as_deref().unwrap_or("flexible");
                 match skill_mode {
                     "sandbox" => {
                         if !docker_available {
                             return Err("Skill requires sandbox but Docker is not available.".to_string());
                         }
                         true
                     },
                     "direct" => false,
                     _ => docker_available, // flexible => use if available
                 }
             }
        };

        // Prepare workspace
        // Use configured workspace path (User CWD)
        let workspace_path = &self.workspace_path;

        // Prepare Skill Files (Code)
        // We write these to a temp dir to keep the user workspace clean, 
        // unless local execution requires them in-place?
        // Docker allows mounting separate paths. Local execution might need them in CWD or accessible.
        
        let skill_files_temp = tempfile::tempdir().map_err(|e| format!("Failed to create temp dir for skill files: {}", e))?;
        let skill_files_path = skill_files_temp.path();

        // Write skill files
        for (rel_path, file) in &self.skill.files {
           let file_path = skill_files_path.join(rel_path);
           if let Some(parent) = file_path.parent() {
               std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create dir: {}", e))?;
           }
           std::fs::write(&file_path, &file.content).map_err(|e| format!("Failed to write file {}: {}", rel_path, e))?;
        }

        let command = if !command_str.is_empty() {
             command_str.to_string()
        } else {
             args.to_string()
        };

        if use_docker {
            let config = self.skill.skill.sandbox_config.clone().unwrap_or(SandboxConfig {
                image: Some("alpine:latest".to_string()),
                memory_limit: Some("128m".to_string()),
                cpu_limit: None,
                timeout_seconds: Some(60),
                network_enabled: Some(false),
            });
            
            // Mount skill files separately to /skill (RO)
            println!("Executing via Docker sandbox...");
            let result = sandbox.execute(&command, workspace_path, Some(skill_files_path), &config).await?;
            if result.success {
                Ok(json!({ "stdout": result.stdout, "stderr": result.stderr }))
            } else {
                Err(format!("Skill execution failed: {}\nStderr: {}", result.stdout, result.stderr))
            }
        } else {
            // Local Execution Fallback
            // For local execution, we need skill files accessible. 
            // If the command relies on "/skill/script.py", it won't work locally.
            // Assumption: Local execution commands are adapted or skills are designed to work relatively.
            // But we can add the skill_files_path to PATH or PYTHONPATH?
            // Or we just strictly run in workspace_path.
            
            let output = tokio::process::Command::new("sh")
                .arg("-c")
                .arg(&command)
                .current_dir(workspace_path)
                .output()
                .await
                .map_err(|e| format!("Failed to execute local command: {}", e))?;
            
            println!("Local execution in workspace: {:?}, command: {}", workspace_path, command);
                
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            
            if output.status.success() {
                 Ok(json!({ "stdout": stdout, "stderr": stderr }))
            } else {
                 Err(format!("Local execution failed: {}\nStderr: {}", stdout, stderr))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ParsedSkill;
    use crate::skills::loader::LoadedSkill;
    use crate::tools::Tool;
    use serde_json::json;
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn create_dummy_skill(requires_sandbox: bool, execution_mode: Option<String>) -> LoadedSkill {
        let parsed = ParsedSkill {
            name: "test".to_string(),
            description: "test".to_string(),
            triggers: None,
            sandbox_config: None,
            body: "".to_string(),
            category: None,
            requires_sandbox,
            license: None,
            execution_mode,
        };
        LoadedSkill {
            skill: parsed,
            files: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_agent_direct_mode_conflict() {
        // Case: Agent says direct, Skill requires sandbox -> Should Fail
        let skill = create_dummy_skill(true, None); // Requires sandbox
        let tool = SkillTool::new(skill, PathBuf::from("."), "direct".to_string());
        
        let ctx: ToolContext<tauri::test::MockRuntime> = ToolContext {
            permissions: std::sync::Arc::new(crate::permissions::PermissionManager::new()),
            window: None,
            session_id: "test".to_string(),
        };

        let result = tool.execute(json!({"args": "echo test"}), &ctx).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Skill requires sandbox but Agent is in 'direct' execution mode.");
    }
}
