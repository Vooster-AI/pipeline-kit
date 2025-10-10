//! Gemini adapter implementation (placeholder for future implementation).
//!
//! This adapter will use JSON-RPC via stdio to communicate with Gemini CLI.
//! For now, it's a placeholder that returns a NotAvailable error.

use crate::agents::base::{Agent, AgentError, AgentEvent, ExecutionContext};
use async_trait::async_trait;
use std::pin::Pin;
use tokio_stream::Stream;

/// Gemini adapter (placeholder).
///
/// TODO: Implement JSON-RPC communication with Gemini CLI.
pub struct GeminiAdapter {
    _name: String,
    _model: String,
    _system_prompt: String,
}

impl GeminiAdapter {
    /// Create a new Gemini adapter.
    pub fn new(name: String, model: String, system_prompt: String) -> Result<Self, AgentError> {
        Ok(Self {
            _name: name,
            _model: model,
            _system_prompt: system_prompt,
        })
    }
}

#[async_trait]
impl Agent for GeminiAdapter {
    async fn check_availability(&self) -> bool {
        // TODO: Check for gemini-cli and GEMINI_API_KEY
        false
    }

    async fn execute(
        &self,
        _context: &ExecutionContext,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<AgentEvent, AgentError>> + Send>>, AgentError> {
        // TODO: Implement JSON-RPC communication
        Err(AgentError::NotAvailable("Gemini adapter not yet implemented".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gemini_adapter_new() {
        let adapter = GeminiAdapter::new(
            "test".to_string(),
            "gemini-2.5-pro".to_string(),
            "test prompt".to_string(),
        );
        assert!(adapter.is_ok());
    }

    #[tokio::test]
    async fn test_check_availability_returns_false() {
        let adapter = GeminiAdapter::new(
            "test".to_string(),
            "gemini-2.5-pro".to_string(),
            "test prompt".to_string(),
        ).unwrap();

        assert!(!adapter.check_availability().await);
    }
}
