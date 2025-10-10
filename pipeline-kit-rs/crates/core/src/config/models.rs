//! Configuration models that aggregate all settings.
//!
//! This module provides the unified `AppConfig` structure that combines
//! global settings, agent definitions, and pipeline definitions into a
//! single configuration object.

use pk_protocol::agent_models::Agent;
use pk_protocol::config_models::GlobalConfig;
use pk_protocol::pipeline_models::Pipeline;

/// Unified application configuration loaded from `.pipeline-kit/` directory.
///
/// This structure aggregates all configuration sources:
/// - `config.toml`: Global settings
/// - `agents/*.md`: Agent definitions
/// - `pipelines/*.yaml`: Pipeline definitions
///
/// # Example
///
/// ```rust,no_run
/// use pk_core::config::loader::load_config;
/// use std::path::Path;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = load_config(Path::new(".")).await?;
/// println!("Loaded {} agents and {} pipelines",
///          config.agents.len(),
///          config.pipelines.len());
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// Global settings from `config.toml`.
    pub global: GlobalConfig,

    /// All agent definitions loaded from `agents/*.md`.
    pub agents: Vec<Agent>,

    /// All pipeline definitions loaded from `pipelines/*.yaml`.
    pub pipelines: Vec<Pipeline>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            global: GlobalConfig { git: false },
            agents: Vec::new(),
            pipelines: Vec::new(),
        }
    }
}
