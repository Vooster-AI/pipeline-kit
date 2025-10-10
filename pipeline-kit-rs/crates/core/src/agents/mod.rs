//! Agent abstraction and management.
//!
//! This module provides the `Agent` trait (Adapter Pattern) and
//! the `AgentManager` for orchestrating multiple agent implementations.

pub mod adapters;
pub mod base;
pub mod manager;

pub use adapters::MockAgent;
pub use base::{Agent, AgentError, AgentEvent, ExecutionContext};
pub use manager::AgentManager;
