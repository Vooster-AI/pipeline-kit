//! # pk-core
//!
//! Core pipeline engine and agent management for pipeline-kit.
//!
//! This crate provides:
//! - Configuration loading from `.pipeline-kit/` directory
//! - Agent abstraction layer and adapter pattern implementation
//! - Pipeline execution engine
//! - State management for running processes
//! - Initialization utilities for creating `.pipeline-kit/` structures
//!
//! ## Modules
//!
//! - [`config`]: Configuration loading and management
//! - [`agents`]: Agent trait and adapter implementations
//! - [`engine`]: Pipeline execution engine
//! - [`state`]: Process state management
//! - [`init`]: Initialization utilities for new projects

pub mod agents;
pub mod config;
pub mod engine;
pub mod init;
pub mod state;
