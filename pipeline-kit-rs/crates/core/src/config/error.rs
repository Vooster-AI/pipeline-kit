//! Error types for configuration loading.
//!
//! This module defines all errors that can occur during configuration file
//! parsing and loading operations.

use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during configuration loading.
#[derive(Error, Debug)]
pub enum ConfigError {
    /// Failed to read a configuration file from disk.
    #[error("Failed to read config file at {path}: {source}")]
    FileRead {
        path: PathBuf,
        source: std::io::Error,
    },

    /// Failed to parse TOML configuration.
    #[error("Failed to parse TOML file at {path}: {source}")]
    TomlParse {
        path: PathBuf,
        source: toml::de::Error,
    },

    /// Failed to parse YAML configuration.
    #[error("Failed to parse YAML file at {path}: {source}")]
    YamlParse {
        path: PathBuf,
        source: serde_yaml::Error,
    },

    /// Failed to parse Markdown front matter.
    #[error("Failed to parse Markdown front matter in {path}: {reason}")]
    MarkdownParse { path: PathBuf, reason: String },

    /// Failed to walk directory structure.
    #[error("Failed to traverse directory {path}: {source}")]
    DirectoryWalk {
        path: PathBuf,
        source: walkdir::Error,
    },

    /// Invalid configuration structure or missing required fields.
    #[error("Invalid configuration in {path}: {reason}")]
    InvalidConfig { path: PathBuf, reason: String },
}

/// Type alias for Result with ConfigError.
pub type ConfigResult<T> = Result<T, ConfigError>;
