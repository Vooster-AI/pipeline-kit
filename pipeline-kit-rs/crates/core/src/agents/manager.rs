//! Agent manager for orchestrating multiple agents.
//!
//! The `AgentManager` is responsible for:
//! - Registering agent configurations
//! - Looking up agents by name
//! - Providing fallback logic when agents are unavailable
//! - Managing the lifecycle of agent instances

use crate::agents::base::{Agent, AgentError, AgentEvent, ExecutionContext};
use pk_protocol::agent_models;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use tokio_stream::Stream;

/// Manages all registered agents and provides orchestration logic.
///
/// The manager maintains a registry of agent adapters and provides
/// methods to look up agents by name and execute instructions with
/// automatic fallback support.
pub struct AgentManager {
    agents: HashMap<String, Arc<dyn Agent>>,
    fallback_agent_name: Option<String>,
}

impl AgentManager {
    /// Create a new AgentManager with the given agent configurations.
    ///
    /// # Arguments
    ///
    /// * `configs` - Agent configurations from `.pipeline-kit/agents/*.md`
    ///
    /// # Returns
    ///
    /// A new `AgentManager` with MockAgent instances for testing.
    ///
    /// # Note
    ///
    /// This is a placeholder implementation. Future tickets will add
    /// support for real agent adapters (Claude CLI, Cursor Agent, etc.).
    pub fn new(configs: Vec<agent_models::Agent>) -> Self {
        let mut agents: HashMap<String, Arc<dyn Agent>> = HashMap::new();

        // For now, create MockAgent instances for each config
        // TODO: Implement factory pattern to create actual agents based on config
        for config in configs {
            let mock_agent = crate::agents::adapters::MockAgent::success();
            agents.insert(config.name.clone(), Arc::new(mock_agent));
        }

        Self {
            agents,
            fallback_agent_name: None,
        }
    }

    /// Set the fallback agent to use when the requested agent is unavailable.
    ///
    /// # Arguments
    ///
    /// * `agent_name` - The name of the agent to use as fallback
    pub fn with_fallback(mut self, agent_name: String) -> Self {
        self.fallback_agent_name = Some(agent_name);
        self
    }

    /// Get an agent by name.
    ///
    /// # Arguments
    ///
    /// * `name` - The agent name to look up
    ///
    /// # Returns
    ///
    /// `Some(Arc<dyn Agent>)` if found, `None` otherwise.
    pub fn get_agent(&self, name: &str) -> Option<Arc<dyn Agent>> {
        self.agents.get(name).cloned()
    }

    /// Execute an instruction with the specified agent.
    ///
    /// This method handles agent lookup and automatic fallback if the
    /// requested agent is unavailable.
    ///
    /// # Arguments
    ///
    /// * `agent_name` - The name of the agent to use
    /// * `context` - The execution context
    ///
    /// # Returns
    ///
    /// A stream of agent events, or an error if no suitable agent is found.
    ///
    /// # Behavior
    ///
    /// 1. Look up the requested agent
    /// 2. Check if it's available
    /// 3. If unavailable and fallback is configured, try fallback agent
    /// 4. Execute with the selected agent
    pub async fn execute(
        &self,
        agent_name: &str,
        context: &ExecutionContext,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<AgentEvent, AgentError>> + Send>>, AgentError> {
        // Try to get the requested agent
        if let Some(agent) = self.get_agent(agent_name) {
            if agent.check_availability().await {
                return agent.execute(context).await;
            }

            // Agent exists but is not available - try fallback
            if let Some(ref fallback_name) = self.fallback_agent_name {
                if fallback_name != agent_name {
                    if let Some(fallback_agent) = self.get_agent(fallback_name) {
                        if fallback_agent.check_availability().await {
                            return fallback_agent.execute(context).await;
                        }
                    }
                }
            }

            return Err(AgentError::NotAvailable(format!(
                "Agent '{}' is not available and no fallback succeeded",
                agent_name
            )));
        }

        Err(AgentError::NotAvailable(format!(
            "Agent '{}' not found in registry",
            agent_name
        )))
    }

    /// List all registered agent names.
    pub fn list_agents(&self) -> Vec<String> {
        self.agents.keys().cloned().collect()
    }

    /// Check if an agent with the given name is registered.
    pub fn has_agent(&self, name: &str) -> bool {
        self.agents.contains_key(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pk_protocol::agent_models::Agent as AgentConfig;
    use tokio_stream::StreamExt;

    fn create_test_config(name: &str) -> AgentConfig {
        AgentConfig {
            name: name.to_string(),
            description: format!("Test agent {}", name),
            model: "test-model".to_string(),
            color: "blue".to_string(),
            system_prompt: "Test prompt".to_string(),
        }
    }

    #[test]
    fn test_agent_manager_new() {
        let configs = vec![
            create_test_config("agent1"),
            create_test_config("agent2"),
        ];

        let manager = AgentManager::new(configs);
        assert!(manager.has_agent("agent1"));
        assert!(manager.has_agent("agent2"));
        assert!(!manager.has_agent("agent3"));
    }

    #[test]
    fn test_agent_manager_get_agent() {
        let configs = vec![create_test_config("test-agent")];
        let manager = AgentManager::new(configs);

        let agent = manager.get_agent("test-agent");
        assert!(agent.is_some());

        let nonexistent = manager.get_agent("nonexistent");
        assert!(nonexistent.is_none());
    }

    #[test]
    fn test_agent_manager_list_agents() {
        let configs = vec![
            create_test_config("agent1"),
            create_test_config("agent2"),
            create_test_config("agent3"),
        ];

        let manager = AgentManager::new(configs);
        let agents = manager.list_agents();

        assert_eq!(agents.len(), 3);
        assert!(agents.contains(&"agent1".to_string()));
        assert!(agents.contains(&"agent2".to_string()));
        assert!(agents.contains(&"agent3".to_string()));
    }

    #[tokio::test]
    async fn test_agent_manager_execute_success() {
        let configs = vec![create_test_config("test-agent")];
        let manager = AgentManager::new(configs);

        let context = ExecutionContext {
            instruction: "test instruction".to_string(),
        };

        let stream = manager.execute("test-agent", &context).await.unwrap();
        let events: Vec<_> = stream.collect().await;

        // MockAgent::success() returns 3 events
        assert_eq!(events.len(), 3);
        assert!(matches!(events[0], Ok(AgentEvent::Thought(_))));
        assert_eq!(events[2], Ok(AgentEvent::Completed));
    }

    #[tokio::test]
    async fn test_agent_manager_execute_not_found() {
        let configs = vec![create_test_config("test-agent")];
        let manager = AgentManager::new(configs);

        let context = ExecutionContext {
            instruction: "test instruction".to_string(),
        };

        let result = manager.execute("nonexistent", &context).await;
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(matches!(e, AgentError::NotAvailable(_)));
        }
    }

    #[tokio::test]
    async fn test_agent_manager_with_fallback() {
        let configs = vec![
            create_test_config("primary"),
            create_test_config("fallback"),
        ];

        let manager = AgentManager::new(configs).with_fallback("fallback".to_string());

        let context = ExecutionContext {
            instruction: "test instruction".to_string(),
        };

        // Primary agent should work
        let stream = manager.execute("primary", &context).await.unwrap();
        let events: Vec<_> = stream.collect().await;
        assert_eq!(events.len(), 3);
    }

    #[test]
    fn test_agent_manager_fallback_configuration() {
        let configs = vec![
            create_test_config("agent1"),
            create_test_config("agent2"),
        ];

        let manager = AgentManager::new(configs).with_fallback("agent2".to_string());
        assert_eq!(manager.fallback_agent_name, Some("agent2".to_string()));
    }
}
