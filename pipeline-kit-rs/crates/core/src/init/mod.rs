//! Initialization module for creating .pipeline-kit directory structures.
//!
//! This module provides functionality to initialize a new Pipeline Kit project
//! by generating a `.pipeline-kit/` directory with pre-configured templates for:
//! - Global configuration (`config.toml`)
//! - Agent definitions (`agents/*.md`)
//! - Pipeline workflows (`pipelines/*.yaml`)
//!
//! # Example
//!
//! ```no_run
//! use pk_core::init::{InitOptions, generate_pipeline_kit_structure};
//! use std::path::PathBuf;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let options = InitOptions {
//!     target_dir: PathBuf::from("."),
//!     force: false,
//!     minimal: false,
//! };
//!
//! generate_pipeline_kit_structure(options).await?;
//! println!("Pipeline Kit initialized successfully!");
//! # Ok(())
//! # }
//! ```

pub mod error;
pub mod generator;
pub mod templates;

// Re-export commonly used types for convenience
pub use error::{InitError, InitResult};
pub use generator::{generate_pipeline_kit_structure, InitOptions};
pub use templates::{get_template, list_templates};
