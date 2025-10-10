//! Error types for initialization operations.

use std::path::PathBuf;
use thiserror::Error;

/// Result type for initialization operations.
pub type InitResult<T> = Result<T, InitError>;

/// Errors that can occur during initialization.
#[derive(Debug, Error)]
pub enum InitError {
    /// The .pipeline-kit directory already exists and force flag was not set.
    #[error(".pipeline-kit directory already exists at {0:?}. Use --force to overwrite.")]
    DirectoryExists(PathBuf),

    /// A required template file was not found in embedded assets.
    #[error("Template file not found: {0}")]
    TemplateNotFound(String),

    /// Failed to create a directory.
    #[error("Failed to create directory {path:?}: {source}")]
    DirectoryCreate {
        path: PathBuf,
        source: std::io::Error,
    },

    /// Failed to write a file.
    #[error("Failed to write file {path:?}: {source}")]
    FileWrite {
        path: PathBuf,
        source: std::io::Error,
    },

    /// Generic I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
