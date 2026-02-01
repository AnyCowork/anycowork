//! SKILL.md parser for parsing skill files with YAML frontmatter
//!
//! Format:
//! ---
//! name: skill-name
//! description: Skill description
//! license: Optional license
//! category: Optional category
//! triggers:
//!   - trigger1
//!   - trigger2
//! requires_sandbox: false
//! sandbox_config:
//!   image: python:3.11
//!   memory_limit: 256m
//!   timeout_seconds: 300
//! ---
//! # Markdown body...

use super::types::{ParsedSkill, SkillSandboxConfig};

/// Parse a SKILL.md file content into a ParsedSkill struct
pub fn parse_skill_md(content: &str) -> Result<ParsedSkill, String> {
    // Check for YAML frontmatter delimiters
    if !content.starts_with("---") {
        return Err("SKILL.md must start with YAML frontmatter (---)".to_string());
    }

    // Find the end of frontmatter
    let content_after_start = &content[3..];
    let end_index = content_after_start
        .find("\n---")
        .ok_or("Could not find end of YAML frontmatter")?;

    let yaml_content = &content_after_start[..end_index].trim();
    let body = content_after_start[end_index + 4..].trim().to_string();

    // Parse YAML frontmatter manually (simple key-value parsing)
    let mut name = String::new();
    let mut description = String::new();
    let mut license: Option<String> = None;
    let mut category: Option<String> = None;
    let mut triggers: Option<Vec<String>> = None;
    let mut requires_sandbox = false;
    let mut sandbox_config: Option<SkillSandboxConfig> = None;
    let mut execution_mode: Option<String> = None;

    let mut in_triggers = false;
    let mut in_sandbox_config = false;
    let mut current_triggers: Vec<String> = Vec::new();
    let mut sandbox_image: Option<String> = None;
    let mut sandbox_memory: Option<String> = None;
    let mut sandbox_cpu: Option<f32> = None;
    let mut sandbox_timeout: Option<u32> = None;
    let mut sandbox_network: Option<bool> = None;

    for line in yaml_content.lines() {
        let trimmed = line.trim();

        // Handle list items under triggers
        if in_triggers && trimmed.starts_with("- ") {
            current_triggers.push(trimmed[2..].trim().to_string());
            continue;
        } else if in_triggers && !trimmed.starts_with("- ") && !trimmed.is_empty() {
            in_triggers = false;
            triggers = Some(current_triggers.clone());
        }

        // Handle sandbox_config nested keys
        if in_sandbox_config {
            if trimmed.starts_with("image:") {
                sandbox_image = Some(extract_value(trimmed, "image:"));
                continue;
            } else if trimmed.starts_with("memory_limit:") {
                sandbox_memory = Some(extract_value(trimmed, "memory_limit:"));
                continue;
            } else if trimmed.starts_with("cpu_limit:") {
                sandbox_cpu = extract_value(trimmed, "cpu_limit:").parse().ok();
                continue;
            } else if trimmed.starts_with("timeout_seconds:") {
                sandbox_timeout = extract_value(trimmed, "timeout_seconds:").parse().ok();
                continue;
            } else if trimmed.starts_with("network_enabled:") {
                let val = extract_value(trimmed, "network_enabled:");
                sandbox_network = Some(val == "true" || val == "yes");
                continue;
            } else if !trimmed.is_empty() && !trimmed.starts_with(' ') {
                // End of sandbox_config section
                in_sandbox_config = false;
                if sandbox_image.is_some()
                    || sandbox_memory.is_some()
                    || sandbox_timeout.is_some()
                {
                    sandbox_config = Some(SkillSandboxConfig {
                        image: sandbox_image.take(),
                        memory_limit: sandbox_memory.take(),
                        cpu_limit: sandbox_cpu.take(),
                        timeout_seconds: sandbox_timeout.take(),
                        network_enabled: sandbox_network.take(),
                    });
                }
            }
        }

        // Parse top-level keys
        if trimmed.starts_with("name:") {
            name = extract_value(trimmed, "name:");
        } else if trimmed.starts_with("description:") {
            description = extract_value(trimmed, "description:");
        } else if trimmed.starts_with("license:") {
            license = Some(extract_value(trimmed, "license:"));
        } else if trimmed.starts_with("category:") {
            category = Some(extract_value(trimmed, "category:"));
        } else if trimmed.starts_with("triggers:") {
            in_triggers = true;
            current_triggers = Vec::new();
        } else if trimmed.starts_with("requires_sandbox:") {
            let val = extract_value(trimmed, "requires_sandbox:");
            requires_sandbox = val == "true" || val == "yes" || val == "1";
        } else if trimmed.starts_with("sandbox_config:") {
            in_sandbox_config = true;
        } else if trimmed.starts_with("execution_mode:") {
            execution_mode = Some(extract_value(trimmed, "execution_mode:"));
        }
    }

    // Handle end-of-file for triggers and sandbox_config
    if in_triggers && !current_triggers.is_empty() {
        triggers = Some(current_triggers);
    }
    if in_sandbox_config
        && (sandbox_image.is_some() || sandbox_memory.is_some() || sandbox_timeout.is_some())
    {
        sandbox_config = Some(SkillSandboxConfig {
            image: sandbox_image,
            memory_limit: sandbox_memory,
            cpu_limit: sandbox_cpu,
            timeout_seconds: sandbox_timeout,
            network_enabled: sandbox_network,
        });
    }

    // Validate required fields
    if name.is_empty() {
        return Err("SKILL.md must have a 'name' field in frontmatter".to_string());
    }
    if description.is_empty() {
        return Err("SKILL.md must have a 'description' field in frontmatter".to_string());
    }

    // Validate name format (hyphen-case, max 64 chars)
    if name.len() > 64 {
        return Err("Skill name must be 64 characters or less".to_string());
    }
    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err(
            "Skill name must only contain alphanumeric characters, hyphens, and underscores"
                .to_string(),
        );
    }

    // Validate description length
    if description.len() > 1024 {
        return Err("Skill description must be 1024 characters or less".to_string());
    }

    Ok(ParsedSkill {
        name,
        description,
        license,
        category,
        triggers,
        requires_sandbox,
        sandbox_config,
        execution_mode,
        body,
    })
}

/// Extract value from a "key: value" line, handling quoted strings
fn extract_value(line: &str, key: &str) -> String {
    let value = line[key.len()..].trim();

    // Remove quotes if present
    if (value.starts_with('"') && value.ends_with('"'))
        || (value.starts_with('\'') && value.ends_with('\''))
    {
        value[1..value.len() - 1].to_string()
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_skill() {
        let content = r#"---
name: test-skill
description: A test skill for demonstration
license: MIT
category: Testing
---
# Test Skill

This is the body of the skill.
"#;

        let result = parse_skill_md(content).unwrap();
        assert_eq!(result.name, "test-skill");
        assert_eq!(result.description, "A test skill for demonstration");
        assert_eq!(result.license, Some("MIT".to_string()));
        assert_eq!(result.category, Some("Testing".to_string()));
        assert!(!result.requires_sandbox);
        assert!(result.body.contains("# Test Skill"));
    }

    #[test]
    fn test_parse_skill_with_sandbox() {
        let content = r#"---
name: sandbox-skill
description: A skill requiring sandbox
requires_sandbox: true
sandbox_config:
  image: python:3.11
  memory_limit: 256m
  timeout_seconds: 300
---
# Sandbox Skill
"#;

        let result = parse_skill_md(content).unwrap();
        assert!(result.requires_sandbox);
        assert!(result.sandbox_config.is_some());

        let config = result.sandbox_config.unwrap();
        assert_eq!(config.image, Some("python:3.11".to_string()));
        assert_eq!(config.memory_limit, Some("256m".to_string()));
        assert_eq!(config.timeout_seconds, Some(300));
    }

    #[test]
    fn test_parse_skill_with_triggers() {
        let content = r#"---
name: triggered-skill
description: A skill with triggers
triggers:
  - pdf
  - document
---
# Triggered Skill
"#;

        let result = parse_skill_md(content).unwrap();
        assert!(result.triggers.is_some());

        let triggers = result.triggers.unwrap();
        assert_eq!(triggers.len(), 2);
        assert!(triggers.contains(&"pdf".to_string()));
        assert!(triggers.contains(&"document".to_string()));
    }

    #[test]
    fn test_missing_name() {
        let content = r#"---
description: Missing name
---
# Body
"#;

        let result = parse_skill_md(content);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("name"));
    }

    #[test]
    fn test_missing_frontmatter() {
        let content = "# No frontmatter\nJust content";
        let result = parse_skill_md(content);
        assert!(result.is_err());
    }
}
