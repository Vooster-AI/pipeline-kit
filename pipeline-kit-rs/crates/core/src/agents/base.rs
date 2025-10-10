//! Base Agent trait and supporting types.

use async_trait::async_trait;
use std::pin::Pin;
use thiserror::Error;
use tokio_stream::Stream;

/// Attachment types that can be included with an instruction.
#[derive(Debug, Clone)]
pub enum Attachment {
    /// Image attachment with path and MIME type.
    Image { path: String, mime_type: String },
    /// File attachment with path and content.
    File { path: String, content: String },
}

/// Context information passed to agents during execution.
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// The user instruction (prompt).
    pub instruction: String,

    /// The project path (working directory).
    pub project_path: String,

    /// Whether this is the initial prompt (used for tool filtering).
    pub is_initial_prompt: bool,

    /// Additional context (images, files, etc.).
    pub attachments: Vec<Attachment>,
}

impl ExecutionContext {
    /// Create a new ExecutionContext with the given instruction.
    ///
    /// Defaults:
    /// - project_path: current directory
    /// - is_initial_prompt: false
    /// - attachments: empty
    pub fn new(instruction: String) -> Self {
        Self {
            instruction,
            project_path: std::env::current_dir()
                .ok()
                .and_then(|p| p.to_str().map(|s| s.to_string()))
                .unwrap_or_else(|| ".".to_string()),
            is_initial_prompt: false,
            attachments: vec![],
        }
    }

    /// Set the project path.
    pub fn with_project_path(mut self, path: String) -> Self {
        self.project_path = path;
        self
    }

    /// Set whether this is an initial prompt.
    pub fn with_initial_prompt(mut self, is_initial: bool) -> Self {
        self.is_initial_prompt = is_initial;
        self
    }

    /// Add an attachment.
    pub fn with_attachment(mut self, attachment: Attachment) -> Self {
        self.attachments.push(attachment);
        self
    }

    /// Add multiple attachments.
    pub fn with_attachments(mut self, attachments: Vec<Attachment>) -> Self {
        self.attachments.extend(attachments);
        self
    }
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
        let context = ExecutionContext::new("test instruction".to_string());

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
        let context = ExecutionContext::new("test instruction".to_string());

        let result = agent.execute(&context).await;
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(matches!(e, AgentError::NotAvailable(_)));
        }
    }

    #[test]
    fn test_execution_context_creation() {
        let context = ExecutionContext::new("Test instruction".to_string());
        assert_eq!(context.instruction, "Test instruction");
        assert!(!context.is_initial_prompt);
        assert!(context.attachments.is_empty());
    }

    #[test]
    fn test_execution_context_builder() {
        let context = ExecutionContext::new("Test".to_string())
            .with_project_path("/tmp/test".to_string())
            .with_initial_prompt(true)
            .with_attachment(Attachment::Image {
                path: "/tmp/image.png".to_string(),
                mime_type: "image/png".to_string(),
            });

        assert_eq!(context.instruction, "Test");
        assert_eq!(context.project_path, "/tmp/test");
        assert!(context.is_initial_prompt);
        assert_eq!(context.attachments.len(), 1);
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
