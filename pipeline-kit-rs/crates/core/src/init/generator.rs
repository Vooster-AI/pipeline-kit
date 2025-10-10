//! Directory structure and file generation for .pipeline-kit initialization.

use super::error::{InitError, InitResult};
use super::templates::{get_template, list_templates};
use std::fs;
use std::path::{Path, PathBuf};

/// Options for initializing a .pipeline-kit directory.
#[derive(Debug, Clone)]
pub struct InitOptions {
    /// Target directory where .pipeline-kit will be created.
    pub target_dir: PathBuf,

    /// Overwrite existing .pipeline-kit directory if it exists.
    pub force: bool,

    /// Create minimal template (only 1 agent and 1 pipeline).
    pub minimal: bool,
}

impl Default for InitOptions {
    fn default() -> Self {
        Self {
            target_dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            force: false,
            minimal: false,
        }
    }
}

/// Generate a complete .pipeline-kit directory structure with templates.
///
/// This function creates the following structure:
/// ```text
/// .pipeline-kit/
/// ├── config.toml
/// ├── agents/
/// │   ├── developer.md
/// │   └── reviewer.md (unless minimal)
/// └── pipelines/
///     ├── simple-task.yaml
///     └── code-review.yaml (unless minimal)
/// ```
///
/// # Arguments
/// * `options` - Configuration for the initialization process
///
/// # Returns
/// `Ok(())` if successful, or an `InitError` if:
/// - The .pipeline-kit directory already exists (without force flag)
/// - A template file cannot be found
/// - File system operations fail
///
/// # Example
/// ```no_run
/// use pk_core::init::{InitOptions, generate_pipeline_kit_structure};
/// use std::path::PathBuf;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let options = InitOptions {
///     target_dir: PathBuf::from("."),
///     force: false,
///     minimal: false,
/// };
///
/// generate_pipeline_kit_structure(options).await?;
/// # Ok(())
/// # }
/// ```
pub async fn generate_pipeline_kit_structure(options: InitOptions) -> InitResult<()> {
    let pk_dir = options.target_dir.join(".pipeline-kit");

    // Check if directory exists
    if pk_dir.exists() && !options.force {
        return Err(InitError::DirectoryExists(pk_dir));
    }

    // Create directory structure
    fs::create_dir_all(pk_dir.join("agents")).map_err(|source| InitError::DirectoryCreate {
        path: pk_dir.join("agents"),
        source,
    })?;

    fs::create_dir_all(pk_dir.join("pipelines")).map_err(|source| InitError::DirectoryCreate {
        path: pk_dir.join("pipelines"),
        source,
    })?;

    // Generate config.toml
    write_template_file(&pk_dir, "config.toml")?;

    // Generate agent templates
    if options.minimal {
        // Only create developer agent
        write_template_file(&pk_dir, "agents/developer.md")?;
    } else {
        // Create all agent templates
        for agent_path in list_templates("agents/") {
            write_template_file(&pk_dir, &agent_path)?;
        }
    }

    // Generate pipeline templates
    if options.minimal {
        // Only create simple-task pipeline
        write_template_file(&pk_dir, "pipelines/simple-task.yaml")?;
    } else {
        // Create all pipeline templates
        for pipeline_path in list_templates("pipelines/") {
            write_template_file(&pk_dir, &pipeline_path)?;
        }
    }

    Ok(())
}

/// Helper function to write a template file to the target directory.
///
/// # Arguments
/// * `pk_dir` - The .pipeline-kit directory path
/// * `template_path` - Relative path of the template (e.g., "agents/developer.md")
///
/// # Returns
/// `Ok(())` if successful, or an `InitError` if the template is not found or writing fails.
fn write_template_file(pk_dir: &Path, template_path: &str) -> InitResult<()> {
    let content = get_template(template_path)
        .ok_or_else(|| InitError::TemplateNotFound(template_path.to_string()))?;

    let target_path = pk_dir.join(template_path);

    // Ensure parent directory exists
    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent).map_err(|source| InitError::DirectoryCreate {
            path: parent.to_path_buf(),
            source,
        })?;
    }

    fs::write(&target_path, content).map_err(|source| InitError::FileWrite {
        path: target_path,
        source,
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    /// RED: This test defines our acceptance criteria and will fail initially.
    ///
    /// We create a complete `.pipeline-kit/` structure and verify all files are correct.
    #[tokio::test]
    async fn test_generate_structure_success() {
        let dir = tempdir().unwrap();
        let options = InitOptions {
            target_dir: dir.path().to_path_buf(),
            force: false,
            minimal: false,
        };

        let result = generate_pipeline_kit_structure(options).await;
        assert!(result.is_ok(), "Failed: {:?}", result.err());

        // Verify directory structure
        let pk_dir = dir.path().join(".pipeline-kit");
        assert!(pk_dir.exists(), ".pipeline-kit directory should exist");
        assert!(
            pk_dir.join("agents").exists(),
            "agents directory should exist"
        );
        assert!(
            pk_dir.join("pipelines").exists(),
            "pipelines directory should exist"
        );

        // Verify config.toml
        assert!(
            pk_dir.join("config.toml").exists(),
            "config.toml should exist"
        );
        let config = fs::read_to_string(pk_dir.join("config.toml")).unwrap();
        assert!(
            config.contains("git ="),
            "config should contain git setting"
        );

        // Verify agents
        assert!(
            pk_dir.join("agents/developer.md").exists(),
            "developer.md should exist"
        );
        assert!(
            pk_dir.join("agents/reviewer.md").exists(),
            "reviewer.md should exist"
        );

        let developer = fs::read_to_string(pk_dir.join("agents/developer.md")).unwrap();
        assert!(
            developer.contains("name: developer"),
            "developer should have correct frontmatter"
        );

        // Verify pipelines
        assert!(
            pk_dir.join("pipelines/simple-task.yaml").exists(),
            "simple-task.yaml should exist"
        );
        assert!(
            pk_dir.join("pipelines/code-review.yaml").exists(),
            "code-review.yaml should exist"
        );

        let simple_task = fs::read_to_string(pk_dir.join("pipelines/simple-task.yaml")).unwrap();
        assert!(
            simple_task.contains("name: simple-task"),
            "simple-task should have correct name"
        );
    }

    /// Test minimal mode generates only essential files.
    #[tokio::test]
    async fn test_generate_structure_minimal() {
        let dir = tempdir().unwrap();
        let options = InitOptions {
            target_dir: dir.path().to_path_buf(),
            force: false,
            minimal: true,
        };

        generate_pipeline_kit_structure(options).await.unwrap();

        let pk_dir = dir.path().join(".pipeline-kit");

        // Should have developer agent
        assert!(
            pk_dir.join("agents/developer.md").exists(),
            "developer.md should exist in minimal mode"
        );

        // Should NOT have reviewer agent
        assert!(
            !pk_dir.join("agents/reviewer.md").exists(),
            "reviewer.md should not exist in minimal mode"
        );

        // Should have simple-task pipeline
        assert!(
            pk_dir.join("pipelines/simple-task.yaml").exists(),
            "simple-task.yaml should exist in minimal mode"
        );

        // Should NOT have code-review pipeline
        assert!(
            !pk_dir.join("pipelines/code-review.yaml").exists(),
            "code-review.yaml should not exist in minimal mode"
        );
    }

    /// Test that existing directory without force flag returns error.
    #[tokio::test]
    async fn test_generate_structure_exists_without_force() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".pipeline-kit")).unwrap();

        let options = InitOptions {
            target_dir: dir.path().to_path_buf(),
            force: false,
            minimal: false,
        };

        let result = generate_pipeline_kit_structure(options).await;
        assert!(result.is_err(), "Should fail when directory exists");
        assert!(
            matches!(result.unwrap_err(), InitError::DirectoryExists(_)),
            "Should return DirectoryExists error"
        );
    }

    /// Test that existing directory with force flag succeeds.
    #[tokio::test]
    async fn test_generate_structure_exists_with_force() {
        let dir = tempdir().unwrap();
        let pk_dir = dir.path().join(".pipeline-kit");
        fs::create_dir_all(&pk_dir).unwrap();

        // Write a file that will be overwritten
        fs::write(pk_dir.join("old-file.txt"), "old content").unwrap();

        let options = InitOptions {
            target_dir: dir.path().to_path_buf(),
            force: true,
            minimal: false,
        };

        let result = generate_pipeline_kit_structure(options).await;
        assert!(result.is_ok(), "Should succeed with force flag");

        // Verify new structure exists
        assert!(
            pk_dir.join("config.toml").exists(),
            "config.toml should be created"
        );
    }

    /// Test default InitOptions.
    #[test]
    fn test_default_init_options() {
        let options = InitOptions::default();
        assert!(!options.force, "Default force should be false");
        assert!(!options.minimal, "Default minimal should be false");
        assert!(
            options.target_dir.is_absolute() || options.target_dir == PathBuf::from("."),
            "Default target_dir should be current directory"
        );
    }
}
