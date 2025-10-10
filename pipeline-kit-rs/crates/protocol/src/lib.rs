//! # pk-protocol
//!
//! Core protocol definitions and data models for pipeline-kit.
//!
//! This crate defines all shared data structures used for:
//! - Configuration file parsing (YAML pipelines, TOML config, Markdown agents)
//! - Runtime process state management
//! - Inter-process communication between TUI and Core
//!
//! ## Modules
//!
//! - [`agent_models`]: Agent configuration structures
//! - [`config_models`]: Global configuration from config.toml
//! - [`pipeline_models`]: Pipeline definitions and process steps
//! - [`process_models`]: Runtime process state and status
//! - [`ipc`]: Operations and Events for Core-TUI communication
//!
//! ## Design Principles
//!
//! - Minimal dependencies: Only serde, ts-rs, and uuid
//! - TypeScript generation: All types derive `TS` for client compatibility
//! - Independent compilation: No dependencies on other pipeline-kit crates

pub mod agent_models;
pub mod config_models;
pub mod ipc;
pub mod pipeline_models;
pub mod process_models;

// Re-export all public types for convenience
pub use agent_models::*;
pub use config_models::*;
pub use ipc::*;
pub use pipeline_models::*;
pub use process_models::*;
