//! Agent abstraction and management.
//!
//! This module provides the `Agent` trait (Adapter Pattern) and
//! the `AgentManager` for orchestrating multiple agent implementations.

pub mod adapters;
pub mod agent_type;
pub mod base;
pub mod factory;
pub mod manager;

pub use adapters::MockAgent;
pub use agent_type::AgentType;
pub use base::Agent;
pub use base::AgentError;
pub use base::AgentEvent;
pub use base::Attachment;
pub use base::ExecutionContext;
pub use factory::AgentFactory;
pub use manager::AgentManager;
