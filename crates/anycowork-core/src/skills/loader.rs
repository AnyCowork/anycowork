//! Skill loader for loading skills from filesystem (directories and ZIP files)

use super::parser::parse_skill_md;
use super::types::ParsedSkill;
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::Path;

/// Loaded skill with all its files
#[derive(Debug, Clone)]
pub struct LoadedSkill {
    pub skill: ParsedSkill,
    pub files: HashMap<String, SkillFileContent>,
}

/// Content of a skill file
#[derive(Debug, Clone)]
pub struct SkillFileContent {
    pub content: String,
    pub file_type: String,
}

/// Load a skill from a directory
///
/// Expected directory structure:
/// skill-name/
///   SKILL.md           # Required - skill definition
///   scripts/           # Optional - executable scripts
///   references/        # Optional - reference documents
///   assets/            # Optional - images, fonts, etc.
///   templates/         # Optional - template files
pub fn load_skill_from_directory(dir_path: &Path) -> Result<LoadedSkill, String> {
    if !dir_path.is_dir() {
        return Err(format!("Path is not a directory: {:?}", dir_path));
    }

    // Read SKILL.md
    let skill_md_path = dir_path.join("SKILL.md");
    if !skill_md_path.exists() {
        return Err(format!("SKILL.md not found in {:?}", dir_path));
    }

    let skill_md_content =
        fs::read_to_string(&skill_md_path).map_err(|e| format!("Failed to read SKILL.md: {}", e))?;

    let skill = parse_skill_md(&skill_md_content)?;

    // Collect additional files from subdirectories
    let mut files: HashMap<String, SkillFileContent> = HashMap::new();

    // Directories to scan for additional files
    let scan_dirs = ["scripts", "references", "assets", "templates", "core"];

    for scan_dir in scan_dirs {
        let sub_path = dir_path.join(scan_dir);
        if sub_path.is_dir() {
            collect_files_recursive(&sub_path, scan_dir, &mut files)?;
        }
    }

    // Also collect any .md files at the root (besides SKILL.md)
    if let Ok(entries) = fs::read_dir(dir_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let file_name = path.file_name().unwrap().to_string_lossy().to_string();
                if file_name.ends_with(".md") && file_name != "SKILL.md" {
                    if let Ok(content) = fs::read_to_string(&path) {
                        files.insert(
                            file_name,
                            SkillFileContent {
                                content,
                                file_type: "markdown".to_string(),
                            },
                        );
                    }
                }
            }
        }
    }

    Ok(LoadedSkill { skill, files })
}

/// Load a skill from a ZIP file
pub fn load_skill_from_zip(zip_path: &Path) -> Result<LoadedSkill, String> {
    let file =
        fs::File::open(zip_path).map_err(|e| format!("Failed to open ZIP file: {}", e))?;

    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| format!("Failed to read ZIP archive: {}", e))?;

    let mut skill_md_content: Option<String> = None;
    let mut files: HashMap<String, SkillFileContent> = HashMap::new();

    // First pass: find SKILL.md and its base directory
    let mut base_dir = String::new();
    for i in 0..archive.len() {
        let file = archive
            .by_index(i)
            .map_err(|e| format!("Failed to read ZIP entry: {}", e))?;
        let name = file.name().to_string();

        if name.ends_with("SKILL.md") {
            base_dir = name.trim_end_matches("SKILL.md").to_string();
            break;
        }
    }

    // Second pass: extract files
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("Failed to read ZIP entry: {}", e))?;

        if file.is_dir() {
            continue;
        }

        let name = file.name().to_string();

        // Skip files outside base directory
        if !base_dir.is_empty() && !name.starts_with(&base_dir) {
            continue;
        }

        // Get relative path from base directory
        let relative_path = if !base_dir.is_empty() {
            name.trim_start_matches(&base_dir).to_string()
        } else {
            name.clone()
        };

        // Read file content
        let mut content = String::new();
        if file.read_to_string(&mut content).is_err() {
            // Binary file, skip for now
            continue;
        }

        if relative_path == "SKILL.md" {
            skill_md_content = Some(content);
        } else if should_include_file(&relative_path) {
            let file_type = detect_file_type(&relative_path);
            files.insert(relative_path, SkillFileContent { content, file_type });
        }
    }

    // Parse SKILL.md
    let skill_md_content = skill_md_content.ok_or("SKILL.md not found in ZIP archive")?;
    let skill = parse_skill_md(&skill_md_content)?;

    Ok(LoadedSkill { skill, files })
}

/// Recursively collect files from a directory
fn collect_files_recursive(
    dir: &Path,
    base_relative: &str,
    files: &mut HashMap<String, SkillFileContent>,
) -> Result<(), String> {
    let entries =
        fs::read_dir(dir).map_err(|e| format!("Failed to read directory {:?}: {}", dir, e))?;

    for entry in entries.flatten() {
        let path = entry.path();
        let file_name = path.file_name().unwrap().to_string_lossy().to_string();
        let relative_path = format!("{}/{}", base_relative, file_name);

        if path.is_dir() {
            collect_files_recursive(&path, &relative_path, files)?;
        } else if path.is_file() && should_include_file(&file_name) {
            match fs::read_to_string(&path) {
                Ok(content) => {
                    let file_type = detect_file_type(&file_name);
                    files.insert(relative_path, SkillFileContent { content, file_type });
                }
                Err(_) => {
                    // Skip binary files or unreadable files
                    log::debug!("Skipping unreadable file: {:?}", path);
                }
            }
        }
    }

    Ok(())
}

/// Determine if a file should be included based on extension
fn should_include_file(filename: &str) -> bool {
    let allowed_extensions = [
        ".py", ".js", ".ts", ".sh", ".bash", ".zsh", // Scripts
        ".md", ".txt", ".rst",     // Documentation
        ".json", ".yaml", ".yml", ".toml", // Config
        ".html", ".css", ".xml", ".xsd", // Web/XML
        ".sql",                     // Database
        ".j2", ".jinja", ".jinja2", // Templates
    ];

    let lowercase = filename.to_lowercase();
    allowed_extensions.iter().any(|ext| lowercase.ends_with(ext))
}

/// Detect file type based on extension
fn detect_file_type(filename: &str) -> String {
    let lowercase = filename.to_lowercase();

    if lowercase.ends_with(".py") {
        "python".to_string()
    } else if lowercase.ends_with(".js") {
        "javascript".to_string()
    } else if lowercase.ends_with(".ts") {
        "typescript".to_string()
    } else if lowercase.ends_with(".sh")
        || lowercase.ends_with(".bash")
        || lowercase.ends_with(".zsh")
    {
        "shell".to_string()
    } else if lowercase.ends_with(".md") {
        "markdown".to_string()
    } else if lowercase.ends_with(".json") {
        "json".to_string()
    } else if lowercase.ends_with(".yaml") || lowercase.ends_with(".yml") {
        "yaml".to_string()
    } else if lowercase.ends_with(".toml") {
        "toml".to_string()
    } else if lowercase.ends_with(".html") {
        "html".to_string()
    } else if lowercase.ends_with(".css") {
        "css".to_string()
    } else if lowercase.ends_with(".xml") || lowercase.ends_with(".xsd") {
        "xml".to_string()
    } else if lowercase.ends_with(".sql") {
        "sql".to_string()
    } else if lowercase.ends_with(".j2")
        || lowercase.ends_with(".jinja")
        || lowercase.ends_with(".jinja2")
    {
        "jinja".to_string()
    } else if lowercase.ends_with(".txt") || lowercase.ends_with(".rst") {
        "text".to_string()
    } else {
        "unknown".to_string()
    }
}

/// List available marketplace skills from a directory
#[allow(dead_code)]
pub fn list_marketplace_skills(skills_dir: &Path) -> Result<Vec<MarketplaceSkillInfo>, String> {
    let mut skills = Vec::new();

    if !skills_dir.is_dir() {
        return Ok(skills);
    }

    let entries = fs::read_dir(skills_dir)
        .map_err(|e| format!("Failed to read skills directory: {}", e))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let skill_md_path = path.join("SKILL.md");
            if skill_md_path.exists() {
                match fs::read_to_string(&skill_md_path) {
                    Ok(content) => match parse_skill_md(&content) {
                        Ok(parsed) => {
                            let dir_name = path
                                .file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_default();
                            skills.push(MarketplaceSkillInfo {
                                name: parsed.name,
                                display_title: parsed.description.chars().take(50).collect::<String>()
                                    + "...",
                                description: parsed.description,
                                category: parsed.category,
                                dir_name,
                                dir_path: path.to_string_lossy().to_string(),
                            });
                        }
                        Err(e) => {
                            log::warn!("Failed to parse SKILL.md in {:?}: {}", path, e);
                        }
                    },
                    Err(e) => {
                        log::warn!("Failed to read SKILL.md in {:?}: {}", path, e);
                    }
                }
            }
        }
    }

    Ok(skills)
}

/// Basic info about a marketplace skill
#[derive(Debug, Clone)]
pub struct MarketplaceSkillInfo {
    pub name: String,
    pub display_title: String,
    pub description: String,
    pub category: Option<String>,
    pub dir_name: String,
    pub dir_path: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_skill_dir() -> TempDir {
        let dir = TempDir::new().unwrap();
        let skill_md = r#"---
name: test-skill
description: A test skill
category: Testing
---
# Test Skill

Body content here.
"#;
        fs::write(dir.path().join("SKILL.md"), skill_md).unwrap();

        // Create scripts directory
        fs::create_dir(dir.path().join("scripts")).unwrap();
        fs::write(dir.path().join("scripts/run.py"), "print('hello')").unwrap();

        dir
    }

    #[test]
    fn test_load_from_directory() {
        let dir = create_test_skill_dir();
        let result = load_skill_from_directory(dir.path()).unwrap();

        assert_eq!(result.skill.name, "test-skill");
        assert!(!result.files.is_empty());
        assert!(result.files.contains_key("scripts/run.py"));
    }

    #[test]
    fn test_detect_file_type() {
        assert_eq!(detect_file_type("test.py"), "python");
        assert_eq!(detect_file_type("test.js"), "javascript");
        assert_eq!(detect_file_type("test.md"), "markdown");
        assert_eq!(detect_file_type("test.sh"), "shell");
    }

    #[test]
    fn test_should_include_file() {
        assert!(should_include_file("script.py"));
        assert!(should_include_file("readme.md"));
        assert!(should_include_file("config.json"));
        assert!(!should_include_file("image.png"));
        assert!(!should_include_file("binary.exe"));
    }
}
