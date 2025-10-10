//! Gemini adapter implementation using JSON-RPC via stdio.
//!
//! This adapter communicates with the Gemini CLI using JSON-RPC protocol
//! over stdin/stdout pipes.

use crate::agents::base::Agent;
use crate::agents::base::AgentError;
use crate::agents::base::AgentEvent;
use crate::agents::base::ExecutionContext;
use async_trait::async_trait;
use serde::Deserialize;
use serde::Serialize;
use std::pin::Pin;
use std::process::Stdio;
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;
use tokio::process::Command;
use tokio_stream::Stream;
use tokio_stream::StreamExt;

/// Gemini adapter for executing instructions using Gemini CLI.
///
/// This adapter spawns the `gemini-cli` process and communicates via JSON-RPC.
pub struct GeminiAdapter {
    #[allow(dead_code)]
    name: String,
    model: String,
    system_prompt: String,
}

impl GeminiAdapter {
    /// Create a new Gemini adapter.
    ///
    /// # Arguments
    ///
    /// * `name` - The agent name from configuration
    /// * `model` - The Gemini model to use (e.g., "gemini-2.5-pro")
    /// * `system_prompt` - The system prompt for the agent
    pub fn new(name: String, model: String, system_prompt: String) -> Result<Self, AgentError> {
        Ok(Self {
            name,
            model,
            system_prompt,
        })
    }
}

#[async_trait]
impl Agent for GeminiAdapter {
    async fn check_availability(&self) -> bool {
        // Check if gemini-cli is installed
        let cli_available = Command::new("gemini-cli")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await
            .map(|s| s.success())
            .unwrap_or(false);

        // Check if GEMINI_API_KEY is set
        let api_key_available = std::env::var("GEMINI_API_KEY").is_ok();

        cli_available && api_key_available
    }

    async fn execute(
        &self,
        context: &ExecutionContext,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<AgentEvent, AgentError>> + Send>>, AgentError>
    {
        // 1. Spawn gemini-cli process with stdin/stdout pipes
        let mut child = Command::new("gemini-cli")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .current_dir(&context.project_path)
            .spawn()
            .map_err(|e| {
                AgentError::ExecutionError(format!("Failed to spawn gemini-cli: {}", e))
            })?;

        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| AgentError::ExecutionError("Failed to capture stdin".to_string()))?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| AgentError::ExecutionError("Failed to capture stdout".to_string()))?;

        // 2. Create JSON-RPC request
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "generate".to_string(),
            params: GenerateParams {
                model: self.model.clone(),
                system: self.system_prompt.clone(),
                prompt: context.instruction.clone(),
            },
        };

        // 3. Send request to Gemini CLI
        let request_str = serde_json::to_string(&request).map_err(|e| {
            AgentError::ExecutionError(format!("Failed to serialize request: {}", e))
        })?;

        stdin
            .write_all(request_str.as_bytes())
            .await
            .map_err(|e| AgentError::ExecutionError(format!("Failed to write to stdin: {}", e)))?;
        stdin
            .write_all(b"\n")
            .await
            .map_err(|e| AgentError::ExecutionError(format!("Failed to write newline: {}", e)))?;
        stdin
            .flush()
            .await
            .map_err(|e| AgentError::ExecutionError(format!("Failed to flush stdin: {}", e)))?;

        // Close stdin to signal end of input
        drop(stdin);

        // 4. Create stream from stdout
        let reader = BufReader::new(stdout);
        let lines = reader.lines();
        let lines_stream = tokio_stream::wrappers::LinesStream::new(lines);

        // 5. Parse JSON-RPC responses and convert to AgentEvents
        let events_stream = lines_stream
            .then(|line_result| async move {
                match line_result {
                    Ok(line) => {
                        if line.trim().is_empty() {
                            return None;
                        }

                        match serde_json::from_str::<JsonRpcResponse>(&line) {
                            Ok(response) => convert_gemini_response(response),
                            Err(e) => Some(Err(AgentError::StreamParseError(format!(
                                "Failed to parse JSON-RPC response: {} (line: {})",
                                e, line
                            )))),
                        }
                    }
                    Err(e) => Some(Err(AgentError::StreamParseError(e.to_string()))),
                }
            })
            .filter_map(|opt| opt);

        Ok(Box::pin(events_stream))
    }
}

/// JSON-RPC request structure.
#[derive(Debug, Serialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: u32,
    method: String,
    params: GenerateParams,
}

/// Parameters for the generate method.
#[derive(Debug, Serialize)]
struct GenerateParams {
    model: String,
    system: String,
    prompt: String,
}

/// JSON-RPC response structure.
#[derive(Debug, Deserialize)]
struct JsonRpcResponse {
    #[allow(dead_code)]
    jsonrpc: String,
    #[allow(dead_code)]
    id: u32,
    result: Option<serde_json::Value>,
    error: Option<JsonRpcError>,
}

/// JSON-RPC error structure.
#[derive(Debug, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

/// Convert Gemini JSON-RPC response to AgentEvent.
fn convert_gemini_response(response: JsonRpcResponse) -> Option<Result<AgentEvent, AgentError>> {
    // Check for errors
    if let Some(error) = response.error {
        return Some(Err(AgentError::ApiError(format!(
            "Gemini API error (code {}): {}",
            error.code, error.message
        ))));
    }

    // Extract result
    if let Some(result) = response.result {
        // Try to extract text from different possible structures

        // Try result.text
        if let Some(text) = result.get("text").and_then(|t| t.as_str()) {
            if !text.trim().is_empty() {
                return Some(Ok(AgentEvent::MessageChunk(text.to_string())));
            }
        }

        // Try result.parts[].text (Google AI format)
        if let Some(parts) = result.get("parts").and_then(|p| p.as_array()) {
            for part in parts {
                if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                    if !text.trim().is_empty() {
                        return Some(Ok(AgentEvent::MessageChunk(text.to_string())));
                    }
                }
            }
        }

        // Try result.content (alternative format)
        if let Some(content) = result.get("content").and_then(|c| c.as_str()) {
            if !content.trim().is_empty() {
                return Some(Ok(AgentEvent::MessageChunk(content.to_string())));
            }
        }

        // If no text found but result exists, signal completion
        return Some(Ok(AgentEvent::Completed));
    }

    // No result and no error - signal completion
    Some(Ok(AgentEvent::Completed))
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
    async fn test_check_availability_without_cli() {
        let adapter = GeminiAdapter::new(
            "test".to_string(),
            "gemini-2.5-pro".to_string(),
            "test prompt".to_string(),
        )
        .unwrap();

        // Will return false unless gemini-cli is installed AND GEMINI_API_KEY is set
        let available = adapter.check_availability().await;
        // We can't assert true/false as it depends on the environment
        let _ = available;
    }

    #[test]
    fn test_json_rpc_request_serialization() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "generate".to_string(),
            params: GenerateParams {
                model: "gemini-2.5-pro".to_string(),
                system: "You are helpful".to_string(),
                prompt: "Hello".to_string(),
            },
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("jsonrpc"));
        assert!(json.contains("generate"));
        assert!(json.contains("gemini-2.5-pro"));
    }

    #[test]
    fn test_convert_gemini_response_with_text() {
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: 1,
            result: Some(serde_json::json!({
                "text": "Hello, world!"
            })),
            error: None,
        };

        let event = convert_gemini_response(response);
        assert!(event.is_some());

        let event = event.unwrap();
        assert!(event.is_ok());

        match event.unwrap() {
            AgentEvent::MessageChunk(text) => assert_eq!(text, "Hello, world!"),
            _ => panic!("Expected MessageChunk"),
        }
    }

    #[test]
    fn test_convert_gemini_response_with_error() {
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: 1,
            result: None,
            error: Some(JsonRpcError {
                code: 400,
                message: "Bad request".to_string(),
            }),
        };

        let event = convert_gemini_response(response);
        assert!(event.is_some());

        let event = event.unwrap();
        assert!(event.is_err());
    }
}
