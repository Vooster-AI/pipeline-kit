//! Base Agent trait and supporting types.

use async_trait::async_trait;
use std::pin::Pin;
use thiserror::Error;
use tokio_stream::Stream;

#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub instruction: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentEvent {
    Thought(String),
    ToolCall(String),
    MessageChunk(String),
    Completed,
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum AgentError {
    #[error("Agent not available: {0}")]
    NotAvailable(String),
    #[error("API call failed: {0}")]
    ApiError(String),
    #[error("Stream parsing error: {0}")]
    StreamParseError(String),
    #[error("Execution failed: {0}")]
    ExecutionError(String),
}

#[async_trait]
pub trait Agent: Send + Sync {
    async fn check_availability(&self) -> bool;
    async fn execute(
        &self,
        context: &ExecutionContext,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<AgentEvent, AgentError>> + Send>>, AgentError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_stream::StreamExt;

    struct TestAgent {
        available: bool,
    }

    #[async_trait]
    impl Agent for TestAgent {
        async fn check_availability(&self) -> bool {
            self.available
        }

        async fn execute(
            &self,
            context: &ExecutionContext,
        ) -> Result<Pin<Box<dyn Stream<Item = Result<AgentEvent, AgentError>> + Send>>, AgentError> {
            if !self.available {
                return Err(AgentError::NotAvailable("Test agent not available".to_string()));
            }

            let instruction = context.instruction.clone();
            let stream = tokio_stream::iter(vec![
                Ok(AgentEvent::Thought(format!("Processing: {}", instruction))),
                Ok(AgentEvent::MessageChunk("Response: ".to_string())),
                Ok(AgentEvent::MessageChunk("Test complete".to_string())),
                Ok(AgentEvent::Completed),
            ]);

            Ok(Box::pin(stream))
        }
    }

    #[tokio::test]
    async fn test_agent_check_availability() {
        let available_agent = TestAgent { available: true };
        assert!(available_agent.check_availability().await);

        let unavailable_agent = TestAgent { available: false };
        assert!(!unavailable_agent.check_availability().await);
    }

    #[tokio::test]
    async fn test_agent_execute_success() {
        let agent = TestAgent { available: true };
        let context = ExecutionContext {
            instruction: "test instruction".to_string(),
        };

        let mut stream = agent.execute(&context).await.unwrap();
        let mut events = Vec::new();

        while let Some(event) = stream.next().await {
            events.push(event.unwrap());
        }

        assert_eq!(events.len(), 4);
        assert!(matches!(events[0], AgentEvent::Thought(_)));
        assert!(matches!(events[1], AgentEvent::MessageChunk(_)));
        assert!(matches!(events[2], AgentEvent::MessageChunk(_)));
        assert_eq!(events[3], AgentEvent::Completed);
    }

    #[tokio::test]
    async fn test_agent_execute_unavailable() {
        let agent = TestAgent { available: false };
        let context = ExecutionContext {
            instruction: "test instruction".to_string(),
        };

        let result = agent.execute(&context).await;
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(matches!(e, AgentError::NotAvailable(_)));
        }
    }

    #[test]
    fn test_execution_context_creation() {
        let context = ExecutionContext {
            instruction: "Test instruction".to_string(),
        };
        assert_eq!(context.instruction, "Test instruction");
    }

    #[test]
    fn test_agent_event_equality() {
        let event1 = AgentEvent::Completed;
        let event2 = AgentEvent::Completed;
        assert_eq!(event1, event2);

        let chunk1 = AgentEvent::MessageChunk("test".to_string());
        let chunk2 = AgentEvent::MessageChunk("test".to_string());
        assert_eq!(chunk1, chunk2);
    }
}
