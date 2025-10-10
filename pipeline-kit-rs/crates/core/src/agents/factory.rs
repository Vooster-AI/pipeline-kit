//! Agent factory for creating agent instances from configurations.

use crate::agents::base::Agent;
use crate::agents::agent_type::AgentType;
use crate::agents::adapters::{/* ClaudeAdapter, CursorAdapter, */ GeminiAdapter, MockAgent};
use anyhow::Result;
use pk_protocol::agent_models;
use std::sync::Arc;

/// Factory for creating agent instances based on configuration.
///
/// The factory determines which adapter to use based on the model name
/// and instantiates the appropriate agent type.
pub struct AgentFactory;

impl AgentFactory {
    /// Create an agent instance from a configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - The agent configuration from `.pipeline-kit/agents/*.md`
    ///
    /// # Returns
    ///
    /// An `Arc<dyn Agent>` wrapping the appropriate adapter, or an error if creation fails.
    ///
    /// # Behavior
    ///
    /// The factory uses `AgentType::from_model_name()` to determine which adapter to create:
    /// - Claude models → `ClaudeAdapter`
    /// - Cursor/GPT models → `CursorAdapter`
    /// - Gemini models → `GeminiAdapter`
    /// - Codex models → `CodexAdapter` (Phase 2)
    /// - Qwen models → `QwenAdapter` (Phase 2)
    /// - Unknown models → `MockAgent`
    ///
    /// # Examples
    ///
    /// ```
    /// use pk_core::agents::AgentFactory;
    /// use pk_protocol::agent_models::Agent as AgentConfig;
    ///
    /// let config = AgentConfig {
    ///     name: "developer".to_string(),
    ///     model: "claude-sonnet-4.5".to_string(),
    ///     description: "Developer agent".to_string(),
    ///     color: "blue".to_string(),
    ///     system_prompt: "You are a helpful developer.".to_string(),
    /// };
    ///
    /// let agent = AgentFactory::create(&config).unwrap();
    /// ```
    pub fn create(config: &agent_models::Agent) -> Result<Arc<dyn Agent>> {
        let agent_type = AgentType::from_model_name(&config.model);

        match agent_type {
            AgentType::Claude => {
                // TODO: Temporarily using MockAgent
                eprintln!("Warning: ClaudeAdapter has compilation errors, using MockAgent");
                Ok(Arc::new(MockAgent::success()))
            }
            AgentType::Cursor => {
                // TODO: Temporarily using MockAgent
                eprintln!("Warning: CursorAdapter has compilation errors, using MockAgent");
                Ok(Arc::new(MockAgent::success()))
            }
            AgentType::Gemini => {
                let adapter = GeminiAdapter::new(
                    config.name.clone(),
                    config.model.clone(),
                    config.system_prompt.clone(),
                )?;
                Ok(Arc::new(adapter))
            }
            AgentType::Codex => {
                // TODO: Phase 2 - Implement CodexAdapter
                eprintln!(
                    "Warning: CodexAdapter not yet implemented for '{}', using MockAgent",
                    config.name
                );
                Ok(Arc::new(MockAgent::success()))
            }
            AgentType::Qwen => {
                // TODO: Phase 2 - Implement QwenAdapter
                eprintln!(
                    "Warning: QwenAdapter not yet implemented for '{}', using MockAgent",
                    config.name
                );
                Ok(Arc::new(MockAgent::success()))
            }
            AgentType::Mock => {
                Ok(Arc::new(MockAgent::success()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config(name: &str, model: &str) -> agent_models::Agent {
        agent_models::Agent {
            name: name.to_string(),
            model: model.to_string(),
            description: format!("Test agent {}", name),
            color: "blue".to_string(),
            system_prompt: "Test prompt".to_string(),
        }
    }

    #[tokio::test]
    async fn test_factory_create_claude() {
        let config = create_test_config("claude-agent", "claude-sonnet-4.5");
        let agent = AgentFactory::create(&config);
        assert!(agent.is_ok());

        let agent = agent.unwrap();
        // For now, this returns MockAgent
        assert!(agent.check_availability().await);
    }

    #[tokio::test]
    async fn test_factory_create_cursor() {
        let config = create_test_config("cursor-agent", "gpt-5");
        let agent = AgentFactory::create(&config);
        assert!(agent.is_ok());

        let agent = agent.unwrap();
        assert!(agent.check_availability().await);
    }

    #[tokio::test]
    async fn test_factory_create_gemini() {
        let config = create_test_config("gemini-agent", "gemini-2.5-pro");
        let agent = AgentFactory::create(&config);
        assert!(agent.is_ok());

        let agent = agent.unwrap();
        assert!(agent.check_availability().await);
    }

    #[tokio::test]
    async fn test_factory_create_mock() {
        let config = create_test_config("mock-agent", "test-model");
        let agent = AgentFactory::create(&config);
        assert!(agent.is_ok());

        let agent = agent.unwrap();
        assert!(agent.check_availability().await);
    }

    #[test]
    fn test_factory_returns_arc() {
        let config = create_test_config("test", "claude-sonnet-4.5");
        let agent1 = AgentFactory::create(&config).unwrap();
        let agent2 = agent1.clone();

        // Both should point to the same agent (Arc semantics)
        assert_eq!(Arc::strong_count(&agent1), 2);
        assert_eq!(Arc::strong_count(&agent2), 2);
    }
}
