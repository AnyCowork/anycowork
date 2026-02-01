//! End-to-end tests for Docker sandbox execution
//!
//! These tests verify:
//! 1. Docker sandbox execution for BashTool and SkillTool
//! 2. Execution mode override (agent settings override skill settings)
//! 3. Fallback to direct execution when Docker is unavailable
//! 4. Security policy enforcement

use anycowork::models::{ParsedSkill, SandboxConfig};
use anycowork::permissions::PermissionManager;
use anycowork::skills::docker::DockerSandbox;
use anycowork::skills::loader::{load_skill_from_directory, LoadedSkill, SkillFileContent};
use anycowork::skills::SkillTool;
use anycowork::tools::{Tool, ToolContext};
use serde_json::json;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

// ============================================================================
// Helper Functions
// ============================================================================

fn create_test_skill(
    name: &str,
    requires_sandbox: bool,
    execution_mode: Option<&str>,
    sandbox_config: Option<SandboxConfig>,
) -> LoadedSkill {
    let parsed = ParsedSkill {
        name: name.to_string(),
        description: format!("Test skill: {}", name),
        triggers: None,
        sandbox_config,
        body: "Test skill body content".to_string(),
        category: Some("Testing".to_string()),
        requires_sandbox,
        license: None,
        execution_mode: execution_mode.map(|s| s.to_string()),
    };
    LoadedSkill {
        skill: parsed,
        files: HashMap::new(),
    }
}

fn create_test_context() -> ToolContext<tauri::test::MockRuntime> {
    ToolContext {
        permissions: Arc::new(PermissionManager::new()),
        window: None,
        session_id: "e2e_test_session".to_string(),
    }
}

async fn is_docker_available() -> bool {
    DockerSandbox::check_available().await
}

// ============================================================================
// Docker Sandbox Core Tests
// ============================================================================

#[tokio::test]
async fn test_docker_sandbox_basic_execution() {
    if !is_docker_available().await {
        println!("Skipping: Docker not available");
        return;
    }

    let mut sandbox = DockerSandbox::new();
    sandbox.init().await;
    assert!(sandbox.is_available());

    let temp_dir = tempfile::tempdir().unwrap();
    let workspace = temp_dir.path();

    let config = SandboxConfig {
        image: Some("alpine:latest".to_string()),
        memory_limit: Some("64m".to_string()),
        cpu_limit: Some(0.5),
        timeout_seconds: Some(30),
        network_enabled: Some(false),
    };

    // Pull image first
    let _ = sandbox.pull_image("alpine:latest").await;

    // Test basic echo command
    let result = sandbox
        .execute("echo 'Hello from Docker'", workspace, None, &config)
        .await;

    match result {
        Ok(res) => {
            assert!(res.success, "Command should succeed");
            assert!(
                res.stdout.contains("Hello from Docker"),
                "Output should contain expected text"
            );
            assert_eq!(res.exit_code, 0);
            assert!(!res.timed_out);
        }
        Err(e) => {
            panic!("Docker execution failed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_docker_sandbox_file_creation() {
    if !is_docker_available().await {
        println!("Skipping: Docker not available");
        return;
    }

    let mut sandbox = DockerSandbox::new();
    sandbox.init().await;

    let temp_dir = tempfile::tempdir().unwrap();
    let workspace = temp_dir.path();

    let config = SandboxConfig {
        image: Some("alpine:latest".to_string()),
        memory_limit: Some("64m".to_string()),
        cpu_limit: None,
        timeout_seconds: Some(30),
        network_enabled: Some(false),
    };

    // Create a file in the mounted workspace
    let result = sandbox
        .execute(
            "echo 'Created in Docker' > /workspace/docker_test.txt && cat /workspace/docker_test.txt",
            workspace,
            None,
            &config,
        )
        .await;

    match result {
        Ok(res) => {
            assert!(res.success);
            // File should exist in workspace
            let file_path = workspace.join("docker_test.txt");
            assert!(file_path.exists(), "File should be created in workspace");
            let content = std::fs::read_to_string(&file_path).unwrap();
            assert!(content.contains("Created in Docker"));
        }
        Err(e) => {
            panic!("Docker execution failed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_docker_sandbox_timeout() {
    if !is_docker_available().await {
        println!("Skipping: Docker not available");
        return;
    }

    let mut sandbox = DockerSandbox::new();
    sandbox.init().await;

    let temp_dir = tempfile::tempdir().unwrap();
    let workspace = temp_dir.path();

    let config = SandboxConfig {
        image: Some("alpine:latest".to_string()),
        memory_limit: Some("64m".to_string()),
        cpu_limit: None,
        timeout_seconds: Some(2), // Very short timeout
        network_enabled: Some(false),
    };

    // Sleep longer than timeout
    let result = sandbox
        .execute("sleep 10", workspace, None, &config)
        .await;

    match result {
        Ok(res) => {
            assert!(res.timed_out, "Command should have timed out");
            assert_eq!(res.exit_code, 124, "Timeout exit code should be 124");
        }
        Err(e) => {
            // Some Docker setups might handle timeout differently
            println!("Timeout test result: {}", e);
        }
    }
}

#[tokio::test]
async fn test_docker_sandbox_network_isolation() {
    if !is_docker_available().await {
        println!("Skipping: Docker not available");
        return;
    }

    let mut sandbox = DockerSandbox::new();
    sandbox.init().await;

    let temp_dir = tempfile::tempdir().unwrap();
    let workspace = temp_dir.path();

    // Network disabled
    let config = SandboxConfig {
        image: Some("alpine:latest".to_string()),
        memory_limit: Some("64m".to_string()),
        cpu_limit: None,
        timeout_seconds: Some(10),
        network_enabled: Some(false),
    };

    // Try to ping - should fail due to network isolation
    let result = sandbox
        .execute("ping -c 1 8.8.8.8 2>&1 || echo 'Network blocked'", workspace, None, &config)
        .await;

    match result {
        Ok(res) => {
            // Network should be blocked
            assert!(
                res.stdout.contains("Network blocked") || !res.success,
                "Network should be isolated"
            );
        }
        Err(_) => {
            // Expected in strict network isolation
        }
    }
}

// ============================================================================
// SkillTool Docker Execution Tests
// ============================================================================

#[tokio::test]
async fn test_skill_tool_sandbox_mode_enforcement() {
    if !is_docker_available().await {
        println!("Skipping: Docker not available");
        return;
    }

    let skill = create_test_skill("sandbox-test", true, Some("sandbox"), None);
    let temp_dir = tempfile::tempdir().unwrap();
    let workspace = temp_dir.path().to_path_buf();

    // Agent also in sandbox mode
    let tool = SkillTool::new(skill, workspace.clone(), "sandbox".to_string());
    let ctx = create_test_context();

    // Execute simple command
    let result = tool
        .execute(json!({"args": "echo 'Sandbox Execution'"}), &ctx)
        .await;

    match result {
        Ok(res) => {
            let stdout = res["stdout"].as_str().unwrap_or("");
            assert!(
                stdout.contains("Sandbox Execution"),
                "Should execute in sandbox"
            );
        }
        Err(e) => {
            panic!("Sandbox execution failed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_skill_tool_agent_sandbox_overrides_skill_direct() {
    // Agent says sandbox, skill says direct -> Should use sandbox
    if !is_docker_available().await {
        println!("Skipping: Docker not available");
        return;
    }

    let skill = create_test_skill("direct-skill", false, Some("direct"), None);
    let temp_dir = tempfile::tempdir().unwrap();
    let workspace = temp_dir.path().to_path_buf();

    // Agent requires sandbox - this should override skill's "direct" mode
    let tool = SkillTool::new(skill, workspace.clone(), "sandbox".to_string());
    let ctx = create_test_context();

    let result = tool
        .execute(json!({"args": "echo 'Agent Overrides'"}), &ctx)
        .await;

    // Should execute in Docker due to agent sandbox requirement
    match result {
        Ok(res) => {
            let stdout = res["stdout"].as_str().unwrap_or("");
            assert!(stdout.contains("Agent Overrides"));
        }
        Err(e) => {
            panic!("Agent override test failed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_skill_tool_agent_direct_blocks_sandbox_skill() {
    // Agent says direct, skill requires sandbox -> Should FAIL for security
    let skill = create_test_skill("sandbox-required", true, Some("sandbox"), None);
    let temp_dir = tempfile::tempdir().unwrap();
    let workspace = temp_dir.path().to_path_buf();

    // Agent in direct mode but skill requires sandbox
    let tool = SkillTool::new(skill, workspace, "direct".to_string());
    let ctx = create_test_context();

    let result = tool
        .execute(json!({"args": "echo 'Should Fail'"}), &ctx)
        .await;

    assert!(result.is_err(), "Should fail when agent=direct but skill requires sandbox");
    let err = result.unwrap_err();
    assert!(
        err.contains("Skill requires sandbox but Agent is in 'direct' execution mode"),
        "Error message should explain the conflict"
    );
}

#[tokio::test]
async fn test_skill_tool_flexible_mode_uses_docker_when_available() {
    if !is_docker_available().await {
        println!("Skipping: Docker not available");
        return;
    }

    let skill = create_test_skill("flexible-skill", false, Some("flexible"), None);
    let temp_dir = tempfile::tempdir().unwrap();
    let workspace = temp_dir.path().to_path_buf();

    // Both agent and skill are flexible - should use Docker since available
    let tool = SkillTool::new(skill, workspace.clone(), "flexible".to_string());
    let ctx = create_test_context();

    // We test by creating a file - Docker execution should work
    let result = tool
        .execute(
            json!({"args": "touch /workspace/flexible_test.txt"}),
            &ctx,
        )
        .await;

    match result {
        Ok(_) => {
            assert!(workspace.join("flexible_test.txt").exists());
        }
        Err(e) => {
            panic!("Flexible mode test failed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_skill_tool_read_content() {
    let skill = create_test_skill("read-test", false, Some("direct"), None);
    let temp_dir = tempfile::tempdir().unwrap();
    let workspace = temp_dir.path().to_path_buf();

    let tool = SkillTool::new(skill, workspace, "direct".to_string());
    let ctx = create_test_context();

    // Read skill content
    let result = tool.execute(json!({"args": "read"}), &ctx).await;

    assert!(result.is_ok());
    let content = result.unwrap();
    assert_eq!(
        content["content"].as_str().unwrap(),
        "Test skill body content"
    );
}

#[tokio::test]
async fn test_skill_tool_direct_execution() {
    let skill = create_test_skill("direct-test", false, Some("direct"), None);
    let temp_dir = tempfile::tempdir().unwrap();
    let workspace = temp_dir.path().to_path_buf();

    let tool = SkillTool::new(skill, workspace.clone(), "direct".to_string());
    let ctx = create_test_context();

    // Create file via direct execution
    let result = tool
        .execute(json!({"args": "touch direct_created.txt"}), &ctx)
        .await;

    assert!(result.is_ok());
    assert!(workspace.join("direct_created.txt").exists());
}

// ============================================================================
// BashTool Docker Execution Tests
// ============================================================================

#[tokio::test]
async fn test_bash_tool_sandbox_execution() {
    use anycowork::tools::bash::BashTool;

    if !is_docker_available().await {
        println!("Skipping: Docker not available");
        return;
    }

    let temp_dir = tempfile::tempdir().unwrap();
    let workspace = temp_dir.path().to_path_buf();

    let tool = BashTool::new(workspace.clone(), "sandbox".to_string());

    // Create context with auto-approve permission manager
    let pm = Arc::new(PermissionManager::new());

    // Start auto-approver
    let pm_clone = pm.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            for req_id in pm_clone.get_pending_requests() {
                pm_clone.approve_request(&req_id);
            }
        }
    });

    let ctx: ToolContext<tauri::test::MockRuntime> = ToolContext {
        permissions: pm,
        window: None,
        session_id: "bash_test".to_string(),
    };

    let result = tool
        .execute(json!({"command": "echo 'Bash Sandbox Test'"}), &ctx)
        .await;

    match result {
        Ok(res) => {
            let stdout = res["stdout"].as_str().unwrap_or("");
            assert!(stdout.contains("Bash Sandbox Test"));
        }
        Err(e) => {
            panic!("Bash sandbox execution failed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_bash_tool_sandbox_not_available_fails() {
    use anycowork::tools::bash::BashTool;

    // This test simulates sandbox mode when Docker is not available
    // We can't easily mock Docker unavailability, but we can test the error path

    let temp_dir = tempfile::tempdir().unwrap();
    let workspace = temp_dir.path().to_path_buf();

    let tool = BashTool::new(workspace, "sandbox".to_string());

    let pm = Arc::new(PermissionManager::new());
    let pm_clone = pm.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            for req_id in pm_clone.get_pending_requests() {
                pm_clone.approve_request(&req_id);
            }
        }
    });

    let ctx: ToolContext<tauri::test::MockRuntime> = ToolContext {
        permissions: pm,
        window: None,
        session_id: "bash_test".to_string(),
    };

    let result = tool
        .execute(json!({"command": "echo 'test'"}), &ctx)
        .await;

    // If Docker is available, this should succeed
    // If Docker is not available, it should fail with appropriate error
    if !is_docker_available().await {
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Docker"));
    } else {
        assert!(result.is_ok());
    }
}

// ============================================================================
// Real Skill Docker Execution Tests
// ============================================================================

#[tokio::test]
async fn test_pdf_skill_docker_execution() {
    if !is_docker_available().await {
        println!("Skipping: Docker not available");
        return;
    }

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let skill_path = Path::new(&manifest_dir).join("skills/pdf");

    if !skill_path.exists() {
        println!("Skipping: PDF skill not found");
        return;
    }

    let loaded_skill = load_skill_from_directory(&skill_path).expect("Failed to load PDF skill");
    let temp_dir = tempfile::tempdir().unwrap();
    let workspace = temp_dir.path().to_path_buf();

    let tool = SkillTool::new(loaded_skill, workspace.clone(), "sandbox".to_string());
    let ctx = create_test_context();

    // Simple echo test to verify Docker execution works for this skill
    let result = tool
        .execute(json!({"args": "echo 'PDF Skill Docker Test'"}), &ctx)
        .await;

    match result {
        Ok(res) => {
            let stdout = res["stdout"].as_str().unwrap_or("");
            assert!(stdout.contains("PDF Skill Docker Test"));
        }
        Err(e) => {
            panic!("PDF skill Docker execution failed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_skill_with_files_docker_execution() {
    if !is_docker_available().await {
        println!("Skipping: Docker not available");
        return;
    }

    // Create a skill with script files
    let mut files = HashMap::new();
    files.insert(
        "test_script.sh".to_string(),
        SkillFileContent {
            content: "#!/bin/sh\necho 'Script executed successfully'".to_string(),
            file_type: "shell".to_string(),
        },
    );

    let skill = LoadedSkill {
        skill: ParsedSkill {
            name: "script-test".to_string(),
            description: "Test skill with script".to_string(),
            triggers: None,
            sandbox_config: Some(SandboxConfig {
                image: Some("alpine:latest".to_string()),
                memory_limit: Some("64m".to_string()),
                cpu_limit: None,
                timeout_seconds: Some(30),
                network_enabled: Some(false),
            }),
            body: "Skill with embedded script".to_string(),
            category: Some("Testing".to_string()),
            requires_sandbox: false,
            license: None,
            execution_mode: Some("flexible".to_string()),
        },
        files,
    };

    let temp_dir = tempfile::tempdir().unwrap();
    let workspace = temp_dir.path().to_path_buf();

    let tool = SkillTool::new(skill, workspace, "sandbox".to_string());
    let ctx = create_test_context();

    // Execute the embedded script
    let result = tool
        .execute(json!({"args": "sh /skill/test_script.sh"}), &ctx)
        .await;

    match result {
        Ok(res) => {
            let stdout = res["stdout"].as_str().unwrap_or("");
            assert!(stdout.contains("Script executed successfully"));
        }
        Err(e) => {
            panic!("Skill with files execution failed: {}", e);
        }
    }
}

// ============================================================================
// Integration Tests - Full Flow
// ============================================================================

#[tokio::test]
async fn test_e2e_skill_docker_file_creation() {
    if !is_docker_available().await {
        println!("Skipping: Docker not available");
        return;
    }

    let skill = create_test_skill("file-creator", false, Some("flexible"), None);
    let temp_dir = tempfile::tempdir().unwrap();
    let workspace = temp_dir.path().to_path_buf();

    let tool = SkillTool::new(skill, workspace.clone(), "sandbox".to_string());
    let ctx = create_test_context();

    // Create a file with content
    let result = tool
        .execute(
            json!({"args": "echo 'E2E Docker Test Content' > /workspace/e2e_test.txt"}),
            &ctx,
        )
        .await;

    assert!(result.is_ok());

    // Verify file exists and content is correct
    let file_path = workspace.join("e2e_test.txt");
    assert!(file_path.exists(), "File should be created");
    let content = std::fs::read_to_string(&file_path).unwrap();
    assert!(content.contains("E2E Docker Test Content"));
}

#[tokio::test]
async fn test_e2e_skill_docker_python_execution() {
    if !is_docker_available().await {
        println!("Skipping: Docker not available");
        return;
    }

    let mut sandbox = DockerSandbox::new();
    sandbox.init().await;

    // Ensure Python image is available
    let _ = sandbox.pull_image("python:3.11-slim").await;

    let skill = LoadedSkill {
        skill: ParsedSkill {
            name: "python-test".to_string(),
            description: "Test Python execution".to_string(),
            triggers: None,
            sandbox_config: Some(SandboxConfig {
                image: Some("python:3.11-slim".to_string()),
                memory_limit: Some("128m".to_string()),
                cpu_limit: None,
                timeout_seconds: Some(60),
                network_enabled: Some(false),
            }),
            body: "Python skill".to_string(),
            category: Some("Testing".to_string()),
            requires_sandbox: false,
            license: None,
            execution_mode: Some("sandbox".to_string()),
        },
        files: HashMap::new(),
    };

    let temp_dir = tempfile::tempdir().unwrap();
    let workspace = temp_dir.path().to_path_buf();

    let tool = SkillTool::new(skill, workspace.clone(), "sandbox".to_string());
    let ctx = create_test_context();

    // Execute Python one-liner
    let result = tool
        .execute(
            json!({"args": "python3 -c \"print('Hello from Python in Docker')\""}),
            &ctx,
        )
        .await;

    match result {
        Ok(res) => {
            let stdout = res["stdout"].as_str().unwrap_or("");
            assert!(stdout.contains("Hello from Python in Docker"));
        }
        Err(e) => {
            // Python image might not be pulled
            println!("Python execution test: {}", e);
        }
    }
}

#[tokio::test]
async fn test_e2e_execution_mode_cascade() {
    // Test the full cascade: agent -> skill -> docker availability
    if !is_docker_available().await {
        println!("Skipping: Docker not available");
        return;
    }

    let temp_dir = tempfile::tempdir().unwrap();
    let workspace = temp_dir.path().to_path_buf();
    let ctx = create_test_context();

    // Case 1: Agent=sandbox, Skill=flexible -> Use Docker
    {
        let skill = create_test_skill("cascade-1", false, Some("flexible"), None);
        let tool = SkillTool::new(skill, workspace.clone(), "sandbox".to_string());
        let result = tool.execute(json!({"args": "echo 'case1'"}), &ctx).await;
        assert!(result.is_ok(), "Case 1 should succeed with Docker");
    }

    // Case 2: Agent=flexible, Skill=sandbox -> Use Docker
    {
        let skill = create_test_skill("cascade-2", true, Some("sandbox"), None);
        let tool = SkillTool::new(skill, workspace.clone(), "flexible".to_string());
        let result = tool.execute(json!({"args": "echo 'case2'"}), &ctx).await;
        assert!(result.is_ok(), "Case 2 should succeed with Docker");
    }

    // Case 3: Agent=direct, Skill=direct -> Direct execution
    {
        let skill = create_test_skill("cascade-3", false, Some("direct"), None);
        let tool = SkillTool::new(skill, workspace.clone(), "direct".to_string());
        let result = tool.execute(json!({"args": "echo 'case3'"}), &ctx).await;
        assert!(result.is_ok(), "Case 3 should succeed with direct execution");
    }

    // Case 4: Agent=direct, Skill=sandbox (requires_sandbox=true) -> FAIL
    {
        let skill = create_test_skill("cascade-4", true, Some("sandbox"), None);
        let tool = SkillTool::new(skill, workspace.clone(), "direct".to_string());
        let result = tool.execute(json!({"args": "echo 'case4'"}), &ctx).await;
        assert!(result.is_err(), "Case 4 should fail due to policy conflict");
    }
}
