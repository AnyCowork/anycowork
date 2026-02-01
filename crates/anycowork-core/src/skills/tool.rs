//! Skill tool for executing skills

use super::loader::LoadedSkill;
use crate::config::ExecutionMode;
use crate::sandbox::{Sandbox, SandboxConfig};
use crate::tools::ToolError;
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Deserialize, JsonSchema)]
pub struct SkillArgs {
    /// Either 'read' to get the full skill guide with code examples, or a shell command to execute. ALWAYS use 'read' first to learn how to use this skill properly.
    pub args: String,
}

#[derive(Serialize, Deserialize)]
pub struct SkillOutput(serde_json::Value);

/// Tool for executing skills
#[derive(Clone)]
pub struct SkillTool {
    /// Skill name
    pub skill_name: String,
    /// Skill description (enhanced)
    pub skill_description: String,
    /// Loaded skill with files
    pub skill: LoadedSkill,
    /// Workspace path
    pub workspace_path: PathBuf,
    /// Execution mode (from agent configuration)
    pub execution_mode: ExecutionMode,
    /// Sandbox for execution
    pub sandbox: Arc<dyn Sandbox>,
}

impl SkillTool {
    /// Create a new skill tool
    pub fn new(
        skill: LoadedSkill,
        workspace_path: PathBuf,
        execution_mode: ExecutionMode,
        sandbox: Arc<dyn Sandbox>,
    ) -> Self {
        // Build an enhanced description that instructs the LLM to read the skill content first
        let enhanced_description = format!(
            "{}. IMPORTANT: Before using this skill, call it with args='read' to get detailed instructions and code examples.",
            skill.skill.description.trim_end_matches('.')
        );

        Self {
            skill_name: skill.skill.name.clone(),
            skill_description: enhanced_description,
            skill,
            workspace_path,
            execution_mode,
            sandbox,
        }
    }

    /// Determine whether to use sandbox based on agent and skill settings
    fn should_use_sandbox(&self, docker_available: bool) -> Result<bool, ToolError> {
        match self.execution_mode {
            ExecutionMode::Sandbox => {
                if !docker_available {
                    return Err(ToolError::execution_failed(
                        "Security Policy Enforcement: Sandbox mode is enabled but Docker is not available.",
                    ));
                }
                Ok(true)
            }
            ExecutionMode::Direct => {
                // If skill requires sandbox, fail in direct mode
                if self.skill.skill.requires_sandbox {
                    return Err(ToolError::execution_failed(
                        "Skill requires sandbox but Agent is in 'direct' execution mode.",
                    ));
                }
                Ok(false)
            }
            ExecutionMode::Flexible => {
                // Fallback to skill preference
                let skill_mode = self
                    .skill
                    .skill
                    .execution_mode
                    .as_deref()
                    .unwrap_or("flexible");
                match skill_mode {
                    "sandbox" => {
                        if !docker_available {
                            return Err(ToolError::execution_failed(
                                "Skill requires sandbox but Docker is not available.",
                            ));
                        }
                        Ok(true)
                    }
                    "direct" => Ok(false),
                    _ => Ok(docker_available), // flexible => use if available
                }
            }
        }
    }
}

impl Tool for SkillTool {
    const NAME: &'static str = "skill_tool"; // Fallback, but we override name() below

    type Error = ToolError;
    type Args = SkillArgs;
    type Output = SkillOutput;

    /// Override name() to return the actual skill name instead of the const NAME
    fn name(&self) -> String {
        self.skill_name.clone()
    }

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: self.skill_name.clone(),
            description: self.skill_description.clone(),
            parameters: serde_json::to_value(schemars::schema_for!(SkillArgs)).unwrap(),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let command_str = args.args;

        // Handle "read" content request for Knowledge skills
        if command_str.trim().eq_ignore_ascii_case("read") {
            return Ok(SkillOutput(json!({ "content": self.skill.skill.body })));
        }

        // Check Docker availability
        let docker_available = self.sandbox.is_available().await;
        let use_docker = self.should_use_sandbox(docker_available)?;

        // Prepare Skill Files (Code)
        let skill_files_temp = tempfile::tempdir()
            .map_err(|e| ToolError::execution_failed(format!("Failed to create temp dir for skill files: {}", e)))?;
        let skill_files_path = skill_files_temp.path();

        // Write skill files
        for (rel_path, file) in &self.skill.files {
            let file_path = skill_files_path.join(rel_path);
            if let Some(parent) = file_path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| ToolError::execution_failed(format!("Failed to create dir: {}", e)))?;
            }
            std::fs::write(&file_path, &file.content)
                .map_err(|e| ToolError::execution_failed(format!("Failed to write file {}: {}", rel_path, e)))?;
        }

        let command = command_str;

        if use_docker {
            // Use skill's sandbox config or defaults
            let config: SandboxConfig = self
                .skill
                .skill
                .sandbox_config
                .clone()
                .map(|c| c.into())
                .unwrap_or_else(|| {
                    SandboxConfig::default()
                        .with_image("alpine:latest")
                        .with_memory_limit("128m")
                        .with_timeout(60)
                });

            // Execute via sandbox with skill files mounted
            log::info!("Executing skill via Docker sandbox...");
            let result = self.sandbox
                .execute_with_files(&command, &self.workspace_path, Some(skill_files_path), &config)
                .await
                .map_err(ToolError::execution_failed)?;

            if result.success {
                Ok(SkillOutput(json!({ "stdout": result.stdout, "stderr": result.stderr })))
            } else {
                Err(ToolError::execution_failed(format!(
                    "Skill execution failed: {}\nStderr: {}",
                    result.stdout, result.stderr
                )))
            }
        } else {
            // Local Execution Fallback
            let config = SandboxConfig::default().with_timeout(60);

            log::info!("Executing skill locally in workspace: {:?}", self.workspace_path);
            let result = self.sandbox
                .execute(&command, &self.workspace_path, &config)
                .await
                .map_err(ToolError::execution_failed)?;

            if result.success {
                Ok(SkillOutput(json!({ "stdout": result.stdout, "stderr": result.stderr })))
            } else {
                Err(ToolError::execution_failed(format!(
                    "Local execution failed: {}\nStderr: {}",
                    result.stdout, result.stderr
                )))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::permissions::AllowAllHandler; // Not used but good to have if needed
    use crate::sandbox::NativeSandbox;
    use crate::skills::types::{ParsedSkill, SkillSandboxConfig};
    use std::collections::HashMap;

    fn create_dummy_skill(requires_sandbox: bool, execution_mode: Option<String>) -> LoadedSkill {
        let parsed = ParsedSkill {
            name: "test".to_string(),
            description: "test".to_string(),
            triggers: None,
            sandbox_config: None,
            body: "Test body".to_string(),
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

    #[test]
    fn test_should_use_sandbox_direct_conflict() {
        // Case: Agent says direct, Skill requires sandbox -> Should Fail
        let skill = create_dummy_skill(true, None);
        let sandbox = Arc::new(NativeSandbox::new());
        let tool = SkillTool::new(skill, PathBuf::from("."), ExecutionMode::Direct, sandbox);

        let result = tool.should_use_sandbox(true);
        assert!(result.is_err());
    }

    #[test]
    fn test_should_use_sandbox_flexible_with_docker() {
        // Case: Flexible mode with Docker available -> Use Docker
        let skill = create_dummy_skill(false, None);
        let sandbox = Arc::new(NativeSandbox::new());
        let tool = SkillTool::new(skill, PathBuf::from("."), ExecutionMode::Flexible, sandbox);

        let result = tool.should_use_sandbox(true).unwrap();
        assert!(result);
    }

    #[test]
    fn test_should_use_sandbox_flexible_without_docker() {
        // Case: Flexible mode without Docker -> Use native
        let skill = create_dummy_skill(false, None);
        let sandbox = Arc::new(NativeSandbox::new());
        let tool = SkillTool::new(skill, PathBuf::from("."), ExecutionMode::Flexible, sandbox);

        let result = tool.should_use_sandbox(false).unwrap();
        assert!(!result);
    }
}
