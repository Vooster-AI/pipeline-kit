//! # pk-core
//!
//! Core pipeline engine and agent management for pipeline-kit.
//!
//! This crate provides:
//! - Configuration loading from `.pipeline-kit/` directory
//! - Agent abstraction layer and adapter pattern implementation
//! - Pipeline execution engine
//! - State management for running processes
//!
//! ## Modules
//!
//! - [`config`]: Configuration loading and management
//! - [`agents`]: Agent trait and adapter implementations
//! - [`engine`]: Pipeline execution engine
//! - [`state`]: Process state management

pub mod agents;
pub mod config;
pub mod engine;
pub mod state;
