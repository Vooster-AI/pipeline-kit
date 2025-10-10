//! Embedded template files for .pipeline-kit initialization.
//!
//! This module uses `rust-embed` to embed template files from the project root
//! `templates/` directory into the binary at compile time. This allows the CLI
//! to generate `.pipeline-kit/` structures without external file dependencies.

use rust_embed::RustEmbed;

/// Embedded template files from the `templates/` directory.
///
/// At compile time, all files in the project root `templates/` directory are
/// embedded into the binary. The path is calculated relative to the crate root:
/// - `CARGO_MANIFEST_DIR` = `pipeline-kit-rs/crates/core`
/// - `../../../templates` = project root `templates/`
///
/// During development with the `debug-embed` feature, files are read from the
/// filesystem at runtime, allowing for quick iteration without recompilation.
#[derive(RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/../../../templates"]
pub struct TemplateAssets;

/// Get template file content by path.
///
/// # Arguments
/// * `path` - Relative path from templates root (e.g., "config.toml", "agents/developer.md")
///
/// # Returns
/// The file content as a String, or None if the file doesn't exist.
///
/// # Example
/// ```
/// use pk_core::init::templates::get_template;
///
/// let config = get_template("config.toml").expect("config.toml should exist");
/// assert!(config.contains("git ="));
/// ```
pub fn get_template(path: &str) -> Option<String> {
    TemplateAssets::get(path).map(|file| String::from_utf8_lossy(file.data.as_ref()).to_string())
}

/// List all template files in a directory.
///
/// # Arguments
/// * `prefix` - Directory prefix (e.g., "agents/", "pipelines/")
///
/// # Returns
/// A vector of file paths that match the prefix.
///
/// # Example
/// ```
/// use pk_core::init::templates::list_templates;
///
/// let agents = list_templates("agents/");
/// assert!(agents.contains(&"agents/developer.md".to_string()));
/// ```
pub fn list_templates(prefix: &str) -> Vec<String> {
    TemplateAssets::iter()
        .filter(|path| path.starts_with(prefix))
        .map(|path| path.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_config_template() {
        let config = get_template("config.toml");
        assert!(config.is_some(), "config.toml should be embedded");
        let content = config.unwrap();
        assert!(
            content.contains("git ="),
            "config.toml should contain git setting"
        );
    }

    #[test]
    fn test_get_agent_template() {
        let developer = get_template("agents/developer.md");
        assert!(
            developer.is_some(),
            "agents/developer.md should be embedded"
        );
        let content = developer.unwrap();
        assert!(
            content.contains("name: developer"),
            "developer.md should have correct frontmatter"
        );
    }

    #[test]
    fn test_get_reviewer_template() {
        let reviewer = get_template("agents/reviewer.md");
        assert!(reviewer.is_some(), "agents/reviewer.md should be embedded");
        let content = reviewer.unwrap();
        assert!(
            content.contains("name: reviewer"),
            "reviewer.md should have correct frontmatter"
        );
    }

    #[test]
    fn test_get_simple_task_pipeline() {
        let pipeline = get_template("pipelines/simple-task.yaml");
        assert!(
            pipeline.is_some(),
            "pipelines/simple-task.yaml should be embedded"
        );
        let content = pipeline.unwrap();
        assert!(
            content.contains("name: simple-task"),
            "simple-task.yaml should have correct name"
        );
    }

    #[test]
    fn test_get_code_review_pipeline() {
        let pipeline = get_template("pipelines/code-review.yaml");
        assert!(
            pipeline.is_some(),
            "pipelines/code-review.yaml should be embedded"
        );
        let content = pipeline.unwrap();
        assert!(
            content.contains("name: code-review"),
            "code-review.yaml should have correct name"
        );
    }

    #[test]
    fn test_get_nonexistent_template() {
        let result = get_template("nonexistent.txt");
        assert!(result.is_none(), "Nonexistent files should return None");
    }

    #[test]
    fn test_list_agent_templates() {
        let agents = list_templates("agents/");
        assert!(!agents.is_empty(), "Should find agent templates");
        assert!(
            agents.contains(&"agents/developer.md".to_string()),
            "Should contain developer.md"
        );
        assert!(
            agents.contains(&"agents/reviewer.md".to_string()),
            "Should contain reviewer.md"
        );
    }

    #[test]
    fn test_list_pipeline_templates() {
        let pipelines = list_templates("pipelines/");
        assert!(!pipelines.is_empty(), "Should find pipeline templates");
        assert!(
            pipelines.contains(&"pipelines/simple-task.yaml".to_string()),
            "Should contain simple-task.yaml"
        );
        assert!(
            pipelines.contains(&"pipelines/code-review.yaml".to_string()),
            "Should contain code-review.yaml"
        );
    }

    #[test]
    fn test_list_empty_prefix() {
        let all = list_templates("");
        assert!(!all.is_empty(), "Should find all templates");
        // Should contain at least config.toml, 2 agents, 2 pipelines
        assert!(all.len() >= 5, "Should have at least 5 template files");
    }
}
