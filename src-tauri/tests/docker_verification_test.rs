use anycowork::skills::loader::load_skill_from_directory;
use anycowork::skills::SkillTool;
use anycowork::tools::Tool;
use anycowork::tools::ToolContext;
use serde_json::json;
use std::path::{Path, PathBuf};

#[tokio::test]
async fn test_verify_docker_execution() {
    // 1. Load PDF skill
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let example_path = Path::new(&manifest_dir).join("skills/pdf");
    
    // Check if skill dir exists (might be missing if not created yet)
    if !example_path.exists() {
        println!("Skipping PDF skill test: directory not found");
        return;
    }

    let loaded_skill = load_skill_from_directory(&example_path).expect("Failed to load PDF skill");

    // 2. Create SkillTool with sandbox enabled (which is default for PDF now)
    // We use a temp dir for workspace
    let temp_dir = tempfile::tempdir().unwrap();
    let workspace_path = temp_dir.path().to_path_buf();
    
    let tool = SkillTool::new(loaded_skill, workspace_path.clone(), "flexible".to_string());

    // 3. Prepare simple command
    // We expect this to run in Docker and print the log we added
    let args = json!({
        "args": "echo 'Docker Verification'"
    });

    let ctx: ToolContext<tauri::test::MockRuntime> = ToolContext {
        permissions: std::sync::Arc::new(anycowork::permissions::PermissionManager::new()),
        window: None,
        session_id: "test_session".to_string(),
    };

    // 4. Execute
    let result = tool.execute(args, &ctx).await;
    
    match result {
        Ok(res) => {
            println!("Execution Result: {:?}", res);
            let stdout = res["stdout"].as_str().unwrap_or("");
            assert!(stdout.contains("Docker Verification"));
        }
        Err(e) => {
            // If Docker is not available, it might fail. 
            // Assert that if it failed, it was due to Docker availability if that's what we expect.
            // But here we want to see SUCCESS with Docker.
            panic!("Execution failed: {}", e);
        }
    }
}
