//! Global configuration models for `.pipeline-kit/config.toml`.
//!
//! This module defines the structure of the global configuration file that
//! controls project-wide settings for pipeline-kit.

use serde::Deserialize;
use serde::Serialize;
use ts_rs::TS;

/// Represents global settings from `.pipeline-kit/config.toml`.
///
/// This structure contains project-wide configuration options that affect
/// all pipeline executions.
///
/// # Example
///
/// ```toml
/// # .pipeline-kit/config.toml
/// git = true
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, TS)]
pub struct GlobalConfig {
    /// Enable git integration for tracking pipeline changes.
    ///
    /// When enabled, pipeline-kit will automatically track changes made
    /// during pipeline execution in git.
    #[serde(default)]
    pub git: bool,
}
