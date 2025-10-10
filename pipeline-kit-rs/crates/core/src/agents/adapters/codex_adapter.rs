//! Codex adapter implementation using OpenAI Codex CLI via JSON-RPC.
//!
//! This adapter communicates with the OpenAI Codex CLI using JSON-RPC protocol
//! over stdin/stdout pipes, following the Python reference implementation.

use crate::agents::base::{Agent, AgentError, AgentEvent, ExecutionContext};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::pin::Pin;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use tokio::fs;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio_stream::Stream;
use tokio_stream::StreamExt;

/// Codex adapter for executing instructions using OpenAI Codex CLI.
///
/// This adapter spawns the `codex` CLI process and communicates via JSON-RPC.
pub struct CodexAdapter {
    name: String,
    model: String,
    system_prompt: String,
    /// Session mapping: project_id -> rollout_file_path
    session_mapping: Arc<Mutex<HashMap<String, String>>>,
}

impl CodexAdapter {
    /// Create a new Codex adapter.
    ///
    /// # Arguments
    ///
    /// * `name` - The agent name from configuration
    /// * `model` - The Codex model to use (e.g., "gpt-4", "codex")
    /// * `system_prompt` - The system prompt for the agent
    pub fn new(name: String, model: String, system_prompt: String) -> Result<Self, AgentError> {
        Ok(Self {
            name,
            model,
            system_prompt,
            session_mapping: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Extract project ID from project path.
    fn extract_project_id(project_path: &str) -> String {
        std::path::Path::new(project_path)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(project_path)
            .to_string()
    }

    /// Ensure AGENTS.md file exists in the project root.
    ///
    /// Codex uses AGENTS.md for system prompts.
    async fn ensure_agent_md(&self, project_path: &str) -> Result<(), AgentError> {
        let agent_md_path = std::path::Path::new(project_path).join("AGENTS.md");

        // Skip if already exists
        if agent_md_path.exists() {
            return Ok(());
        }

        // Write system prompt to AGENTS.md
        fs::write(&agent_md_path, &self.system_prompt)
            .await
            .map_err(|e| AgentError::ExecutionError(format!("Failed to write AGENTS.md: {}", e)))?;

        Ok(())
    }

    /// Get or create rollout file path for session management.
    async fn get_rollout_path(
        &self,
        project_path: &str,
        project_id: &str,
    ) -> Result<String, AgentError> {
        // Create rollout directory: .pipeline-kit/codex_rollouts/
        let rollout_dir = std::path::Path::new(project_path)
            .join(".pipeline-kit")
            .join("codex_rollouts");

        fs::create_dir_all(&rollout_dir)
            .await
            .map_err(|e| {
                AgentError::ExecutionError(format!("Failed to create rollout directory: {}", e))
            })?;

        // Rollout file: <project_id>.yaml
        let rollout_file = rollout_dir.join(format!("{}.yaml", project_id));
        Ok(rollout_file.to_string_lossy().to_string())
    }

    /// Determine the codex executable name based on platform.
    fn get_executable_name() -> &'static str {
        if cfg!(target_os = "windows") {
            "codex.cmd"
        } else {
            "codex"
        }
    }
}

#[async_trait]
impl Agent for CodexAdapter {
    async fn check_availability(&self) -> bool {
        // Check if codex CLI is installed
        let cli_available = Command::new(Self::get_executable_name())
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await
            .map(|s| s.success())
            .unwrap_or(false);

        // Check if OPENAI_API_KEY is set
        let api_key_available = std::env::var("OPENAI_API_KEY").is_ok();

        cli_available && api_key_available
    }

    async fn execute(
        &self,
        context: &ExecutionContext,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<AgentEvent, AgentError>> + Send>>, AgentError> {
        // 1. Ensure AGENTS.md exists
        self.ensure_agent_md(&context.project_path).await?;

        // 2. Get rollout file path
        let project_id = Self::extract_project_id(&context.project_path);
        let rollout_path = self.get_rollout_path(&context.project_path, &project_id).await?;

        // 3. Build command
        let mut cmd = Command::new(Self::get_executable_name());
        cmd.arg("--model").arg(&self.model);
        cmd.arg("--approval-policy").arg("allow-all"); // Auto-approve all actions
        cmd.arg("--rollout").arg(&rollout_path); // Session persistence
        cmd.arg("--output-format").arg("jsonrpc"); // JSON-RPC output

        // API key from environment
        if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
            cmd.env("OPENAI_API_KEY", api_key);
        }

        // Set working directory
        cmd.current_dir(&context.project_path);
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        // 4. Spawn process
        let mut child = cmd.spawn().map_err(|e| {
            AgentError::ExecutionError(format!("Failed to spawn codex CLI: {}", e))
        })?;

        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| AgentError::ExecutionError("Failed to capture stdin".to_string()))?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| AgentError::ExecutionError("Failed to capture stdout".to_string()))?;

        // 5. Create JSON-RPC request
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "execute".to_string(),
            params: ExecuteParams {
                prompt: context.instruction.clone(),
                system: self.system_prompt.clone(),
            },
        };

        // 6. Send request to Codex CLI
        let request_str = serde_json::to_string(&request)
            .map_err(|e| AgentError::ExecutionError(format!("Failed to serialize request: {}", e)))?;

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

        // 7. Create stream from stdout
        let reader = BufReader::new(stdout);
        let lines = reader.lines();
        let lines_stream = tokio_stream::wrappers::LinesStream::new(lines);

        // 8. Save rollout path for this session
        {
            let mut mapping = self.session_mapping.lock().unwrap();
            mapping.insert(project_id.clone(), rollout_path);
        }

        // 9. Parse JSON-RPC responses and convert to AgentEvents
        let events_stream = lines_stream
            .then(|line_result| async move {
                match line_result {
                    Ok(line) => {
                        if line.trim().is_empty() {
                            return None;
                        }

                        match serde_json::from_str::<JsonRpcResponse>(&line) {
                            Ok(response) => convert_codex_response(response),
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
    params: ExecuteParams,
}

/// Parameters for the execute method.
#[derive(Debug, Serialize)]
struct ExecuteParams {
    prompt: String,
    system: String,
}

/// JSON-RPC response structure.
#[derive(Debug, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: u32,
    result: Option<CodexResult>,
    error: Option<JsonRpcError>,
}

/// Codex result structure.
#[derive(Debug, Deserialize)]
struct CodexResult {
    #[serde(rename = "type")]
    event_type: String,
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    tool_event: Option<ToolEvent>,
}

/// Tool event structure.
#[derive(Debug, Deserialize)]
struct ToolEvent {
    #[serde(rename = "type")]
    tool_type: String,
    #[serde(default)]
    command: Option<String>,
    #[serde(default)]
    patch: Option<String>,
    #[serde(default)]
    query: Option<String>,
}

/// JSON-RPC error structure.
#[derive(Debug, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

/// Convert Codex JSON-RPC response to AgentEvent.
fn convert_codex_response(
    response: JsonRpcResponse,
) -> Option<Result<AgentEvent, AgentError>> {
    // Check for errors
    if let Some(error) = response.error {
        return Some(Err(AgentError::ApiError(format!(
            "Codex API error (code {}): {}",
            error.code, error.message
        ))));
    }

    // Extract result
    if let Some(result) = response.result {
        match result.event_type.as_str() {
            "message" => {
                // Text message from assistant
                if let Some(content) = result.content {
                    if !content.trim().is_empty() {
                        return Some(Ok(AgentEvent::MessageChunk(content)));
                    }
                }
            }
            "tool_event" => {
                // Tool execution event
                if let Some(tool_event) = result.tool_event {
                    let tool_json = match tool_event.tool_type.as_str() {
                        "exec_command" => {
                            let cmd = tool_event.command.unwrap_or_default();
                            serde_json::json!({
                                "type": "exec_command",
                                "command": cmd
                            })
                        }
                        "patch_apply" => {
                            let patch = tool_event.patch.unwrap_or_default();
                            serde_json::json!({
                                "type": "patch_apply",
                                "patch": patch
                            })
                        }
                        "web_search" => {
                            let query = tool_event.query.unwrap_or_default();
                            serde_json::json!({
                                "type": "web_search",
                                "query": query
                            })
                        }
                        "mcp_tool_call" => {
                            serde_json::json!({
                                "type": "mcp_tool_call"
                            })
                        }
                        _ => {
                            serde_json::json!({
                                "type": "unknown",
                                "tool_type": tool_event.tool_type
                            })
                        }
                    };

                    return Some(Ok(AgentEvent::ToolCall(tool_json.to_string())));
                }
            }
            "done" | "completed" => {
                // Execution completed
                return Some(Ok(AgentEvent::Completed));
            }
            _ => {
                // Unknown event type, skip
            }
        }
    }

    // No meaningful event
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codex_adapter_new() {
        let adapter = CodexAdapter::new(
            "test".to_string(),
            "gpt-4".to_string(),
            "test prompt".to_string(),
        );
        assert!(adapter.is_ok());
    }

    #[test]
    fn test_extract_project_id() {
        assert_eq!(CodexAdapter::extract_project_id("/path/to/project"), "project");
        assert_eq!(CodexAdapter::extract_project_id("/tmp/test"), "test");
    }

    #[tokio::test]
    async fn test_check_availability_without_cli() {
        let adapter = CodexAdapter::new(
            "test".to_string(),
            "gpt-4".to_string(),
            "test prompt".to_string(),
        )
        .unwrap();

        // Will return false unless codex CLI is installed AND OPENAI_API_KEY is set
        let available = adapter.check_availability().await;
        let _ = available;
    }

    #[test]
    fn test_json_rpc_request_serialization() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "execute".to_string(),
            params: ExecuteParams {
                prompt: "Hello".to_string(),
                system: "You are helpful".to_string(),
            },
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("jsonrpc"));
        assert!(json.contains("execute"));
        assert!(json.contains("Hello"));
    }

    #[test]
    fn test_convert_codex_response_with_message() {
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: 1,
            result: Some(CodexResult {
                event_type: "message".to_string(),
                content: Some("Hello, world!".to_string()),
                tool_event: None,
            }),
            error: None,
        };

        let event = convert_codex_response(response);
        assert!(event.is_some());

        let event = event.unwrap();
        assert!(event.is_ok());

        match event.unwrap() {
            AgentEvent::MessageChunk(text) => assert_eq!(text, "Hello, world!"),
            _ => panic!("Expected MessageChunk"),
        }
    }

    #[test]
    fn test_convert_codex_response_with_tool_event() {
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: 1,
            result: Some(CodexResult {
                event_type: "tool_event".to_string(),
                content: None,
                tool_event: Some(ToolEvent {
                    tool_type: "exec_command".to_string(),
                    command: Some("ls -la".to_string()),
                    patch: None,
                    query: None,
                }),
            }),
            error: None,
        };

        let event = convert_codex_response(response);
        assert!(event.is_some());

        let event = event.unwrap();
        assert!(event.is_ok());

        match event.unwrap() {
            AgentEvent::ToolCall(json) => {
                assert!(json.contains("exec_command"));
                assert!(json.contains("ls -la"));
            }
            _ => panic!("Expected ToolCall"),
        }
    }

    #[test]
    fn test_convert_codex_response_with_completion() {
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: 1,
            result: Some(CodexResult {
                event_type: "done".to_string(),
                content: None,
                tool_event: None,
            }),
            error: None,
        };

        let event = convert_codex_response(response);
        assert!(event.is_some());

        let event = event.unwrap();
        assert!(event.is_ok());

        match event.unwrap() {
            AgentEvent::Completed => {}
            _ => panic!("Expected Completed"),
        }
    }

    #[test]
    fn test_convert_codex_response_with_error() {
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: 1,
            result: None,
            error: Some(JsonRpcError {
                code: 400,
                message: "Bad request".to_string(),
            }),
        };

        let event = convert_codex_response(response);
        assert!(event.is_some());

        let event = event.unwrap();
        assert!(event.is_err());
    }

    #[tokio::test]
    async fn test_ensure_agent_md() {
        let adapter = CodexAdapter::new(
            "test".to_string(),
            "gpt-4".to_string(),
            "test system prompt".to_string(),
        )
        .unwrap();

        let temp_dir = tempfile::tempdir().unwrap();
        let project_path = temp_dir.path().to_str().unwrap();

        let result = adapter.ensure_agent_md(project_path).await;
        assert!(result.is_ok());

        // Verify file was created
        let agent_md_path = temp_dir.path().join("AGENTS.md");
        assert!(agent_md_path.exists());

        let content = fs::read_to_string(&agent_md_path).await.unwrap();
        assert_eq!(content, "test system prompt");

        // Calling again should not error (idempotent)
        let result2 = adapter.ensure_agent_md(project_path).await;
        assert!(result2.is_ok());
    }

    #[tokio::test]
    async fn test_get_rollout_path() {
        let adapter = CodexAdapter::new(
            "test".to_string(),
            "gpt-4".to_string(),
            "test prompt".to_string(),
        )
        .unwrap();

        let temp_dir = tempfile::tempdir().unwrap();
        let project_path = temp_dir.path().to_str().unwrap();

        let rollout_path = adapter
            .get_rollout_path(project_path, "my-project")
            .await
            .unwrap();

        assert!(rollout_path.contains(".pipeline-kit"));
        assert!(rollout_path.contains("codex_rollouts"));
        assert!(rollout_path.contains("my-project.yaml"));

        // Verify directory was created
        let rollout_dir = temp_dir.path().join(".pipeline-kit").join("codex_rollouts");
        assert!(rollout_dir.exists());
    }

    #[test]
    fn test_get_executable_name() {
        let exe_name = CodexAdapter::get_executable_name();
        if cfg!(target_os = "windows") {
            assert_eq!(exe_name, "codex.cmd");
        } else {
            assert_eq!(exe_name, "codex");
        }
    }
}
