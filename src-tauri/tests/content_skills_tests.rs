use anyagents::skills::loader::load_skill_from_directory;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_validate_all_content_skills() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let skills_root = PathBuf::from(manifest_dir).join("skills");

    if !skills_root.exists() {
        // If the directory doesn't exist (e.g. in some CI envs where we might restrict checkout),
        // we might skip or fail. Since the user asked for tests *for* these skills, presumably they exist.
        // Let's assert it exists.
        panic!("Skills root directory not found at: {:?}", skills_root);
    }

    let entries = fs::read_dir(&skills_root).expect("Failed to read skills root directory");
    let mut failure_count = 0;
    let mut success_count = 0;
    let mut failures = Vec::new();

    for entry in entries {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();

        if path.is_dir() {
            let dir_name = path.file_name().unwrap().to_string_lossy().to_string();
            println!("Testing skill: {}", dir_name);

            match load_skill_from_directory(&path) {
                Ok(loaded_skill) => {
                    println!("  ✓ Loaded '{}' successfully", loaded_skill.skill.name);
                    success_count += 1;
                    
                    // Additional validation could go here
                    // e.g. check if triggers are valid, if docker image is valid format, etc.
                }
                Err(e) => {
                    println!("  ✗ Failed to load '{}': {}", dir_name, e);
                    failure_count += 1;
                    failures.push((dir_name, e));
                }
            }
        }
    }

    println!("\nSummary:");
    println!("  Total Scanned: {}", success_count + failure_count);
    println!("  Passed: {}", success_count);
    println!("  Failed: {}", failure_count);

    if failure_count > 0 {
        println!("\nFailures:");
        for (name, error) in failures {
            println!("  - {}: {}", name, error);
        }
        panic!("Validation failed for {} skills", failure_count);
    }
}
