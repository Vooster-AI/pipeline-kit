//! Mock agent implementations for deterministic testing.

use async_trait::async_trait;
use pk_core::agents::base::Agent;
use pk_core::agents::base::AgentError;
use pk_core::agents::base::AgentEvent;
use pk_core::agents::base::ExecutionContext;
use std::pin::Pin;
use tokio_stream::Stream;

/// A mock agent that always succeeds with a predefined response.
#[allow(dead_code)]
pub struct MockSuccessAgent {
    pub name: String,
    pub response: String,
}

impl MockSuccessAgent {
    #[allow(dead_code)]
    pub fn new(name: &str, response: &str) -> Self {
        Self {
            name: name.to_string(),
            response: response.to_string(),
        }
    }
}

#[async_trait]
impl Agent for MockSuccessAgent {
    async fn check_availability(&self) -> bool {
        true
    }

    async fn execute(
        &self,
        _context: &ExecutionContext,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<AgentEvent, AgentError>> + Send>>, AgentError>
    {
        let response = self.response.clone();

        // Create a simple stream that emits the response and completes
        let stream = async_stream::stream! {
            yield Ok(AgentEvent::MessageChunk(response));
            yield Ok(AgentEvent::Completed);
        };

        Ok(Box::pin(stream))
    }
}

/// A mock agent that always fails with a predefined error.
#[allow(dead_code)]
pub struct MockFailureAgent {
    pub name: String,
    pub error_message: String,
}

impl MockFailureAgent {
    #[allow(dead_code)]
    pub fn new(name: &str, error_message: &str) -> Self {
        Self {
            name: name.to_string(),
            error_message: error_message.to_string(),
        }
    }
}

#[async_trait]
impl Agent for MockFailureAgent {
    async fn check_availability(&self) -> bool {
        true
    }

    async fn execute(
        &self,
        _context: &ExecutionContext,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<AgentEvent, AgentError>> + Send>>, AgentError>
    {
        let error_message = self.error_message.clone();

        let stream = async_stream::stream! {
            yield Err(AgentError::ExecutionError(error_message));
        };

        Ok(Box::pin(stream))
    }
}

/// A mock agent that delays before responding.
#[allow(dead_code)]
pub struct MockDelayedAgent {
    pub name: String,
    pub response: String,
    pub delay_ms: u64,
}

impl MockDelayedAgent {
    #[allow(dead_code)]
    pub fn new(name: &str, response: &str, delay_ms: u64) -> Self {
        Self {
            name: name.to_string(),
            response: response.to_string(),
            delay_ms,
        }
    }
}

#[async_trait]
impl Agent for MockDelayedAgent {
    async fn check_availability(&self) -> bool {
        true
    }

    async fn execute(
        &self,
        _context: &ExecutionContext,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<AgentEvent, AgentError>> + Send>>, AgentError>
    {
        let response = self.response.clone();
        let delay_ms = self.delay_ms;

        let stream = async_stream::stream! {
            tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
            yield Ok(AgentEvent::MessageChunk(response));
            yield Ok(AgentEvent::Completed);
        };

        Ok(Box::pin(stream))
    }
}
