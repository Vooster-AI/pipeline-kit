//! Configuration file loader for `.pipeline-kit/` directory structure.
//!
//! This module provides functionality to load and parse all configuration files
//! from the `.pipeline-kit/` directory, including:
//! - `config.toml`: Global settings
//! - `agents/*.md`: Agent definitions with YAML front matter
//! - `pipelines/*.yaml`: Pipeline definitions

use crate::config::error::ConfigError;
use crate::config::error::ConfigResult;
use crate::config::models::AppConfig;
use gray_matter::engine::YAML;
use gray_matter::Matter;
use pk_protocol::agent_models::Agent;
use pk_protocol::config_models::GlobalConfig;
use pk_protocol::pipeline_models::Pipeline;
use std::path::Path;
use walkdir::WalkDir;

/// Loads all configuration from the `.pipeline-kit/` directory.
///
/// This function scans the `.pipeline-kit/` directory and loads:
/// - Global configuration from `config.toml`
/// - Agent definitions from `agents/*.md` files
/// - Pipeline definitions from `pipelines/*.yaml` files
///
/// # Arguments
///
/// * `root` - Root directory containing the `.pipeline-kit/` folder
///
/// # Returns
///
/// An `AppConfig` containing all loaded configuration. If directories or files
/// are missing (but the root exists), returns an empty/default configuration
/// rather than an error.
///
/// # Errors
///
/// Returns `ConfigError` if:
/// - Files exist but cannot be read
/// - Files have invalid syntax (TOML, YAML, or Markdown front matter)
/// - Required fields are missing in configuration files
///
/// # Example
///
/// ```rust,no_run
/// use pk_core::config::loader::load_config;
/// use std::path::Path;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = load_config(Path::new(".")).await?;
/// println!("Loaded {} agents", config.agents.len());
/// # Ok(())
/// # }
/// ```
pub async fn load_config(root: &Path) -> ConfigResult<AppConfig> {
    let pk_dir = root.join(".pipeline-kit");

    // If .pipeline-kit doesn't exist, return default config
    if !pk_dir.exists() {
        return Ok(AppConfig::default());
    }

    // Load global config
    let global = load_global_config(&pk_dir)?;

    // Load agents
    let agents = load_agents(&pk_dir)?;

    // Load pipelines
    let pipelines = load_pipelines(&pk_dir)?;

    Ok(AppConfig {
        global,
        agents,
        pipelines,
    })
}

/// Loads global configuration from `config.toml`.
fn load_global_config(pk_dir: &Path) -> ConfigResult<GlobalConfig> {
    let config_path = pk_dir.join("config.toml");

    // If config.toml doesn't exist, return default
    if !config_path.exists() {
        return Ok(GlobalConfig { git: false });
    }

    let content =
        std::fs::read_to_string(&config_path).map_err(|source| ConfigError::FileRead {
            path: config_path.clone(),
            source,
        })?;

    let config: GlobalConfig =
        toml::from_str(&content).map_err(|source| ConfigError::TomlParse {
            path: config_path,
            source,
        })?;

    Ok(config)
}

/// Loads all agent definitions from `agents/*.md`.
fn load_agents(pk_dir: &Path) -> ConfigResult<Vec<Agent>> {
    let agents_dir = pk_dir.join("agents");

    // If agents directory doesn't exist, return empty vector
    if !agents_dir.exists() {
        return Ok(Vec::new());
    }

    let mut agents = Vec::new();

    // Walk through all .md files in the agents directory
    for entry in WalkDir::new(&agents_dir)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
    {
        let entry = entry.map_err(|source| ConfigError::DirectoryWalk {
            path: agents_dir.clone(),
            source,
        })?;

        let path = entry.path();

        // Only process .md files
        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }

        let content = std::fs::read_to_string(path).map_err(|source| ConfigError::FileRead {
            path: path.to_path_buf(),
            source,
        })?;

        // Parse Markdown with YAML front matter
        let matter = Matter::<YAML>::new();
        let result = matter.parse(&content);

        let mut agent: Agent = result
            .data
            .ok_or_else(|| ConfigError::MarkdownParse {
                path: path.to_path_buf(),
                reason: "Missing YAML front matter".to_string(),
            })?
            .deserialize()
            .map_err(|e| ConfigError::MarkdownParse {
                path: path.to_path_buf(),
                reason: format!("Failed to deserialize front matter: {}", e),
            })?;

        // Set the system prompt from the markdown body
        agent.system_prompt = result.content;

        agents.push(agent);
    }

    Ok(agents)
}

/// Loads all pipeline definitions from `pipelines/*.yaml`.
fn load_pipelines(pk_dir: &Path) -> ConfigResult<Vec<Pipeline>> {
    let pipelines_dir = pk_dir.join("pipelines");

    // If pipelines directory doesn't exist, return empty vector
    if !pipelines_dir.exists() {
        return Ok(Vec::new());
    }

    let mut pipelines = Vec::new();

    // Walk through all .yaml and .yml files in the pipelines directory
    for entry in WalkDir::new(&pipelines_dir)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
    {
        let entry = entry.map_err(|source| ConfigError::DirectoryWalk {
            path: pipelines_dir.clone(),
            source,
        })?;

        let path = entry.path();

        // Only process .yaml and .yml files
        let ext = path.extension().and_then(|s| s.to_str());
        if ext != Some("yaml") && ext != Some("yml") {
            continue;
        }

        let content = std::fs::read_to_string(path).map_err(|source| ConfigError::FileRead {
            path: path.to_path_buf(),
            source,
        })?;

        let pipeline: Pipeline =
            serde_yaml::from_str(&content).map_err(|source| ConfigError::YamlParse {
                path: path.to_path_buf(),
                source,
            })?;

        pipelines.push(pipeline);
    }

    Ok(pipelines)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    /// RED: This test defines our acceptance criteria and will fail initially.
    ///
    /// We create a complete `.pipeline-kit/` structure with all required files
    /// and verify that `load_config` correctly parses and loads everything.
    #[tokio::test]
    async fn test_load_config_acceptance() {
        // Setup: Create temporary .pipeline-kit directory structure
        let dir = tempdir().expect("Failed to create temp dir");
        let root = dir.path();
        let pk_dir = root.join(".pipeline-kit");

        fs::create_dir_all(pk_dir.join("pipelines")).expect("Failed to create pipelines dir");
        fs::create_dir_all(pk_dir.join("agents")).expect("Failed to create agents dir");

        // Write config.toml
        let config_toml = "git = true";
        fs::write(pk_dir.join("config.toml"), config_toml).expect("Failed to write config.toml");

        // Write an agent definition (Markdown with YAML front matter)
        let agent_md = r#"---
name: code-reviewer
description: Reviews code for quality
model: claude-sonnet-4
color: blue
---

You are an expert code reviewer. Analyze code for:
- Correctness
- Performance
- Security"#;
        fs::write(pk_dir.join("agents/code-reviewer.md"), agent_md)
            .expect("Failed to write agent file");

        // Write a pipeline definition
        let pipeline_yaml = r#"name: review-pipeline
required-reference-file:
  1: "docs/standards.md"
output-file:
  1: "review-report.md"
master:
  model: claude-sonnet-4
  system-prompt: "Orchestrate code review"
  process:
    - code-reviewer
    - HUMAN_REVIEW
sub-agents:
  - code-reviewer
"#;
        fs::write(pk_dir.join("pipelines/review.yaml"), pipeline_yaml)
            .expect("Failed to write pipeline file");

        // Act: Load configuration (this will fail until we implement it)
        let config = load_config(root).await.expect("Failed to load config");

        // Assert: Verify all configuration was loaded correctly

        // Global config
        assert!(config.global.git, "Global git setting should be true");

        // Agents
        assert_eq!(config.agents.len(), 1, "Should load 1 agent");
        let agent = &config.agents[0];
        assert_eq!(agent.name, "code-reviewer");
        assert_eq!(agent.description, "Reviews code for quality");
        assert_eq!(agent.model, "claude-sonnet-4");
        assert_eq!(agent.color, "blue");
        assert!(
            agent.system_prompt.contains("expert code reviewer"),
            "System prompt should be loaded from markdown body"
        );

        // Pipelines
        assert_eq!(config.pipelines.len(), 1, "Should load 1 pipeline");
        let pipeline = &config.pipelines[0];
        assert_eq!(pipeline.name, "review-pipeline");
        assert_eq!(
            pipeline.required_reference_file.get(&1),
            Some(&"docs/standards.md".to_string())
        );
        assert_eq!(
            pipeline.output_file.get(&1),
            Some(&"review-report.md".to_string())
        );
        assert_eq!(pipeline.master.model, "claude-sonnet-4");
        assert_eq!(pipeline.master.process.len(), 2);
        assert_eq!(pipeline.sub_agents.len(), 1);
        assert_eq!(pipeline.sub_agents[0], "code-reviewer");
    }

    /// RED: Test loading from an empty directory (no .pipeline-kit folder).
    ///
    /// This should return a default/empty configuration, not an error.
    #[tokio::test]
    async fn test_load_config_empty_directory() {
        let dir = tempdir().expect("Failed to create temp dir");
        let root = dir.path();

        // No .pipeline-kit directory exists
        let config = load_config(root)
            .await
            .expect("Should handle missing .pipeline-kit");

        // Should return empty/default configuration
        assert!(!config.global.git, "Default git should be false");
        assert!(config.agents.is_empty(), "Should have no agents");
        assert!(config.pipelines.is_empty(), "Should have no pipelines");
    }

    /// RED: Test partial configuration (only config.toml exists).
    #[tokio::test]
    async fn test_load_config_partial() {
        let dir = tempdir().expect("Failed to create temp dir");
        let root = dir.path();
        let pk_dir = root.join(".pipeline-kit");

        fs::create_dir_all(&pk_dir).expect("Failed to create .pipeline-kit");

        // Only write config.toml
        fs::write(pk_dir.join("config.toml"), "git = false").expect("Failed to write config.toml");

        let config = load_config(root)
            .await
            .expect("Should handle partial config");

        assert!(!config.global.git);
        assert!(config.agents.is_empty(), "Should have no agents");
        assert!(config.pipelines.is_empty(), "Should have no pipelines");
    }

    /// REFACTOR: Test invalid TOML syntax.
    #[tokio::test]
    async fn test_load_config_invalid_toml() {
        let dir = tempdir().expect("Failed to create temp dir");
        let root = dir.path();
        let pk_dir = root.join(".pipeline-kit");

        fs::create_dir_all(&pk_dir).expect("Failed to create .pipeline-kit");

        // Write invalid TOML
        fs::write(pk_dir.join("config.toml"), "git = [invalid toml")
            .expect("Failed to write config.toml");

        let result = load_config(root).await;
        assert!(result.is_err(), "Should fail on invalid TOML");

        if let Err(ConfigError::TomlParse { path, .. }) = result {
            assert!(path.ends_with("config.toml"));
        } else {
            panic!("Expected TomlParse error");
        }
    }

    /// REFACTOR: Test invalid YAML in pipeline file.
    #[tokio::test]
    async fn test_load_config_invalid_yaml() {
        let dir = tempdir().expect("Failed to create temp dir");
        let root = dir.path();
        let pk_dir = root.join(".pipeline-kit");

        fs::create_dir_all(pk_dir.join("pipelines")).expect("Failed to create pipelines dir");

        // Write invalid YAML
        let invalid_yaml = "name: test\n  invalid: [yaml";
        fs::write(pk_dir.join("pipelines/test.yaml"), invalid_yaml)
            .expect("Failed to write pipeline file");

        let result = load_config(root).await;
        assert!(result.is_err(), "Should fail on invalid YAML");

        if let Err(ConfigError::YamlParse { path, .. }) = result {
            assert!(path.ends_with("test.yaml"));
        } else {
            panic!("Expected YamlParse error");
        }
    }

    /// REFACTOR: Test agent markdown file without front matter.
    #[tokio::test]
    async fn test_load_config_agent_no_frontmatter() {
        let dir = tempdir().expect("Failed to create temp dir");
        let root = dir.path();
        let pk_dir = root.join(".pipeline-kit");

        fs::create_dir_all(pk_dir.join("agents")).expect("Failed to create agents dir");

        // Write markdown without front matter
        let no_frontmatter = "Just plain markdown content";
        fs::write(pk_dir.join("agents/test.md"), no_frontmatter)
            .expect("Failed to write agent file");

        let result = load_config(root).await;
        assert!(result.is_err(), "Should fail on agent without front matter");

        if let Err(ConfigError::MarkdownParse { path, reason }) = result {
            assert!(path.ends_with("test.md"));
            assert!(reason.contains("Missing YAML front matter"));
        } else {
            panic!("Expected MarkdownParse error");
        }
    }

    /// REFACTOR: Test agent markdown file with invalid front matter.
    #[tokio::test]
    async fn test_load_config_agent_invalid_frontmatter() {
        let dir = tempdir().expect("Failed to create temp dir");
        let root = dir.path();
        let pk_dir = root.join(".pipeline-kit");

        fs::create_dir_all(pk_dir.join("agents")).expect("Failed to create agents dir");

        // Write markdown with incomplete front matter (missing required fields)
        let invalid_frontmatter = r#"---
name: test-agent
# Missing required fields: description, model
---

Agent content"#;
        fs::write(pk_dir.join("agents/test.md"), invalid_frontmatter)
            .expect("Failed to write agent file");

        let result = load_config(root).await;
        assert!(
            result.is_err(),
            "Should fail on agent with invalid front matter"
        );

        if let Err(ConfigError::MarkdownParse { path, reason }) = result {
            assert!(path.ends_with("test.md"));
            assert!(reason.contains("Failed to deserialize"));
        } else {
            panic!("Expected MarkdownParse error");
        }
    }

    /// REFACTOR: Test loading multiple agents and pipelines.
    #[tokio::test]
    async fn test_load_config_multiple_files() {
        let dir = tempdir().expect("Failed to create temp dir");
        let root = dir.path();
        let pk_dir = root.join(".pipeline-kit");

        fs::create_dir_all(pk_dir.join("agents")).expect("Failed to create agents dir");
        fs::create_dir_all(pk_dir.join("pipelines")).expect("Failed to create pipelines dir");

        // Write multiple agent files
        for i in 1..=3 {
            let agent_md = format!(
                r#"---
name: agent-{}
description: Test agent {}
model: test-model
color: blue
---

System prompt for agent {}"#,
                i, i, i
            );
            fs::write(pk_dir.join(format!("agents/agent-{}.md", i)), agent_md)
                .expect("Failed to write agent file");
        }

        // Write multiple pipeline files
        for i in 1..=2 {
            let pipeline_yaml = format!(
                r#"name: pipeline-{}
master:
  model: test-model
  system-prompt: "Test prompt"
  process:
    - agent-1
sub-agents:
  - agent-1
"#,
                i
            );
            fs::write(
                pk_dir.join(format!("pipelines/pipeline-{}.yaml", i)),
                pipeline_yaml,
            )
            .expect("Failed to write pipeline file");
        }

        let config = load_config(root).await.expect("Should load multiple files");

        assert_eq!(config.agents.len(), 3, "Should load 3 agents");
        assert_eq!(config.pipelines.len(), 2, "Should load 2 pipelines");
    }

    /// REFACTOR: Test that non-matching files are ignored.
    #[tokio::test]
    async fn test_load_config_ignores_non_matching_files() {
        let dir = tempdir().expect("Failed to create temp dir");
        let root = dir.path();
        let pk_dir = root.join(".pipeline-kit");

        fs::create_dir_all(pk_dir.join("agents")).expect("Failed to create agents dir");
        fs::create_dir_all(pk_dir.join("pipelines")).expect("Failed to create pipelines dir");

        // Write files with wrong extensions
        fs::write(pk_dir.join("agents/readme.txt"), "Not a markdown file")
            .expect("Failed to write txt file");
        fs::write(pk_dir.join("pipelines/notes.txt"), "Not a yaml file")
            .expect("Failed to write txt file");

        // Write one valid file
        let agent_md = r#"---
name: valid-agent
description: Valid agent
model: test-model
---

Valid content"#;
        fs::write(pk_dir.join("agents/valid.md"), agent_md).expect("Failed to write agent file");

        let config = load_config(root)
            .await
            .expect("Should ignore non-matching files");

        assert_eq!(config.agents.len(), 1, "Should only load .md files");
        assert_eq!(config.pipelines.len(), 0, "Should only load .yaml files");
    }

    /// REFACTOR: Test loading with .yml extension (alternative to .yaml).
    #[tokio::test]
    async fn test_load_config_yml_extension() {
        let dir = tempdir().expect("Failed to create temp dir");
        let root = dir.path();
        let pk_dir = root.join(".pipeline-kit");

        fs::create_dir_all(pk_dir.join("pipelines")).expect("Failed to create pipelines dir");

        // Write pipeline with .yml extension
        let pipeline_yaml = r#"name: yml-pipeline
master:
  model: test-model
  system-prompt: "Test"
  process:
    - test-agent
sub-agents:
  - test-agent
"#;
        fs::write(pk_dir.join("pipelines/test.yml"), pipeline_yaml)
            .expect("Failed to write pipeline file");

        let config = load_config(root).await.expect("Should load .yml files");

        assert_eq!(config.pipelines.len(), 1, "Should load .yml files");
        assert_eq!(config.pipelines[0].name, "yml-pipeline");
    }
}
