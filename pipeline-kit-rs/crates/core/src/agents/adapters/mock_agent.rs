//! Mock agent implementation for testing.

use crate::agents::base::{Agent, AgentError, AgentEvent, ExecutionContext};
use async_trait::async_trait;
use std::pin::Pin;
use tokio_stream::Stream;

#[derive(Clone)]
pub struct MockAgent {
    available: bool,
    events: Vec<Result<AgentEvent, AgentError>>,
}

impl MockAgent {
    pub fn new(available: bool, events: Vec<Result<AgentEvent, AgentError>>) -> Self {
        Self { available, events }
    }

    pub fn success() -> Self {
        Self {
            available: true,
            events: vec![
                Ok(AgentEvent::Thought("Mock agent thinking".to_string())),
                Ok(AgentEvent::MessageChunk("Mock response".to_string())),
                Ok(AgentEvent::Completed),
            ],
        }
    }

    pub fn unavailable() -> Self {
        Self {
            available: false,
            events: vec![],
        }
    }

    pub fn failing() -> Self {
        Self {
            available: true,
            events: vec![
                Ok(AgentEvent::Thought("Starting...".to_string())),
                Err(AgentError::ExecutionError("Mock failure".to_string())),
            ],
        }
    }
}

#[async_trait]
impl Agent for MockAgent {
    async fn check_availability(&self) -> bool {
        self.available
    }

    async fn execute(
        &self,
        _context: &ExecutionContext,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<AgentEvent, AgentError>> + Send>>, AgentError> {
        if !self.available {
            return Err(AgentError::NotAvailable("Mock agent not available".to_string()));
        }

        let events = self.events.clone();
        let stream = tokio_stream::iter(events);
        Ok(Box::pin(stream))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_stream::StreamExt;

    #[tokio::test]
    async fn test_mock_agent_success() {
        let agent = MockAgent::success();
        assert!(agent.check_availability().await);

        let context = ExecutionContext {
            instruction: "test".to_string(),
        };

        let mut stream = agent.execute(&context).await.unwrap();
        let mut events = Vec::new();

        while let Some(event) = stream.next().await {
            events.push(event);
        }

        assert_eq!(events.len(), 3);
        assert!(matches!(events[0], Ok(AgentEvent::Thought(_))));
        assert!(matches!(events[1], Ok(AgentEvent::MessageChunk(_))));
        assert_eq!(events[2], Ok(AgentEvent::Completed));
    }

    #[tokio::test]
    async fn test_mock_agent_unavailable() {
        let agent = MockAgent::unavailable();
        assert!(!agent.check_availability().await);

        let context = ExecutionContext {
            instruction: "test".to_string(),
        };

        let result = agent.execute(&context).await;
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(matches!(e, AgentError::NotAvailable(_)));
        }
    }

    #[tokio::test]
    async fn test_mock_agent_failing() {
        let agent = MockAgent::failing();
        assert!(agent.check_availability().await);

        let context = ExecutionContext {
            instruction: "test".to_string(),
        };

        let stream = agent.execute(&context).await.unwrap();
        let events: Vec<_> = stream.collect().await;

        assert_eq!(events.len(), 2);
        assert!(matches!(events[0], Ok(AgentEvent::Thought(_))));
        assert!(matches!(events[1], Err(AgentError::ExecutionError(_))));
    }

    #[tokio::test]
    async fn test_mock_agent_custom_events() {
        let custom_events = vec![
            Ok(AgentEvent::ToolCall("read_file".to_string())),
            Ok(AgentEvent::MessageChunk("File content".to_string())),
            Ok(AgentEvent::Completed),
        ];

        let agent = MockAgent::new(true, custom_events);
        let context = ExecutionContext {
            instruction: "test".to_string(),
        };

        let stream = agent.execute(&context).await.unwrap();
        let events: Vec<_> = stream.collect().await;

        assert_eq!(events.len(), 3);
        assert!(matches!(events[0], Ok(AgentEvent::ToolCall(_))));
    }
}
