//! Qwen adapter implementation using Qwen CLI via JSON-RPC (ACP Protocol).
//!
//! This is an MVP implementation following a simplified architecture:
//! - Uses notification-only JSON-RPC (no response matching)
//! - No reverse request handlers (permissions ignored for now)
//! - No buffering (immediate event yielding)
//! - Fresh session per execution
//!
//! Future phases will add:
//! - Phase 2: Permission auto-approval, session reuse
//! - Phase 3: Buffer management, complete reverse request handlers

use crate::agents::base::{Agent, AgentError, AgentEvent, ExecutionContext};
use async_trait::async_trait;
// Allow: Serialize will be used in Phase 2 for JSON-RPC request serialization
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};
use std::pin::Pin;
// Allow: Stdio, tokio types will be used in Phase 2 for process spawning and I/O
#[allow(unused_imports)]
use std::process::Stdio;
#[allow(unused_imports)]
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
#[allow(unused_imports)]
use tokio::process::{Child, Command};
use tokio_stream::Stream;

/// Minimal JSON-RPC notification structure (no id field = notification).
// Allow: Will be used in Phase 2 for parsing stdout notifications
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct Notification {
    method: String,
    params: serde_json::Value,
}

/// Session update parameters from session/update notifications.
// Allow: Will be used in Phase 2 for parsing session/update notifications
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct SessionUpdate {
    #[serde(rename = "sessionUpdate")]
    kind: Option<String>,
    #[serde(rename = "type")]
    type_field: Option<String>,
    text: Option<String>,
    content: Option<serde_json::Value>,
}

/// Qwen adapter for executing instructions using Qwen CLI.
///
/// This adapter spawns the `qwen` CLI process with `--experimental-acp` flag
/// and communicates via JSON-RPC protocol over stdin/stdout.
pub struct QwenAdapter {
    #[allow(dead_code)]
    // Allow: Public API field, may be used by external consumers
    name: String,
    // Allow: Will be used in Phase 2 for qwen CLI invocation
    #[allow(dead_code)]
    model: String,
    // Allow: Will be used in Phase 2 for QWEN.md content and initialization
    #[allow(dead_code)]
    system_prompt: String,
}

impl QwenAdapter {
    /// Create a new Qwen adapter.
    ///
    /// # Arguments
    ///
    /// * `name` - The agent name from configuration
    /// * `model` - The Qwen model to use (e.g., "qwen-coder", "qwen2.5-coder")
    /// * `system_prompt` - The system prompt for the agent
    pub fn new(name: String, model: String, system_prompt: String) -> Result<Self, AgentError> {
        Ok(Self {
            name,
            model,
            system_prompt,
        })
    }

    /// Resolve the qwen CLI command.
    ///
    /// Checks in the following order:
    /// 1. QWEN_CMD environment variable
    /// 2. `qwen` in PATH
    /// 3. `qwen-code` in PATH
    fn resolve_qwen_command(&self) -> Result<String, AgentError> {
        // Check QWEN_CMD environment variable
        if let Ok(cmd) = std::env::var("QWEN_CMD") {
            if which::which(&cmd).is_ok() {
                return Ok(cmd);
            }
        }

        // Check standard command names
        for cmd in &["qwen", "qwen-code"] {
            if which::which(cmd).is_ok() {
                return Ok(cmd.to_string());
            }
        }

        Err(AgentError::NotAvailable(
            "Qwen CLI not found. Install 'qwen' or set QWEN_CMD environment variable".to_string(),
        ))
    }

    /// Ensure QWEN.md file exists in the project root.
    ///
    /// Qwen CLI uses QWEN.md for system prompts.
    async fn ensure_qwen_md(&self, project_path: &str) -> Result<(), AgentError> {
        let qwen_md_path = std::path::Path::new(project_path).join("QWEN.md");

        // Skip if already exists
        if qwen_md_path.exists() {
            return Ok(());
        }

        // Write system prompt to QWEN.md with a header
        let content = format!("# QWEN\n\n{}", self.system_prompt);
        tokio::fs::write(&qwen_md_path, content)
            .await
            .map_err(|e| AgentError::ExecutionError(format!("Failed to create QWEN.md: {}", e)))?;

        Ok(())
    }

    /// Send a JSON-RPC request to stdin (response is not awaited in MVP).
    async fn send_request(
        stdin: &mut tokio::process::ChildStdin,
        request: &serde_json::Value,
    ) -> Result<(), AgentError> {
        let json = serde_json::to_string(request)
            .map_err(|e| AgentError::ExecutionError(format!("JSON serialize error: {}", e)))?;

        stdin
            .write_all(json.as_bytes())
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

        Ok(())
    }

    /// Create a stream that reads notifications from stdout and converts them to AgentEvents.
    fn create_notification_stream(
        stdout: tokio::process::ChildStdout,
    ) -> impl Stream<Item = Result<AgentEvent, AgentError>> {
        async_stream::stream! {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();

            while let Ok(Some(line)) = lines.next_line().await {
                if line.trim().is_empty() {
                    continue;
                }

                // Parse JSON
                let value: serde_json::Value = match serde_json::from_str(&line) {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("[qwen] Failed to parse JSON: {} (line: {})", e, line);
                        continue;
                    }
                };

                // Only process notifications (has method, no id)
                if value.get("method").is_some() && value.get("id").is_none() {
                    let notification: Notification = match serde_json::from_value(value) {
                        Ok(n) => n,
                        Err(e) => {
                            eprintln!("[qwen] Failed to parse notification: {}", e);
                            continue;
                        }
                    };

                    // Only handle session/update notifications
                    if notification.method == "session/update" {
                        if let Some(event) = Self::convert_update(notification.params) {
                            yield Ok(event);
                        }
                    }
                }
                // Ignore requests/responses (reverse requests are not handled in MVP)
            }

            yield Ok(AgentEvent::Completed);
        }
    }

    /// Convert a session/update notification to an AgentEvent.
    fn convert_update(params: serde_json::Value) -> Option<AgentEvent> {
        // Clone params before parsing to allow reuse for tool_call
        let params_clone = params.clone();
        let update: SessionUpdate = serde_json::from_value(params).ok()?;

        // Extract kind from either `sessionUpdate` or `type` field
        let kind = update.kind.or(update.type_field)?;

        match kind.as_str() {
            "agent_message_chunk" | "agent_thought_chunk" => {
                // Extract text content
                let text = update.text.or_else(|| {
                    update
                        .content
                        .as_ref()
                        .and_then(|c| c.get("text"))
                        .and_then(|t| t.as_str())
                        .map(|s| s.to_string())
                })?;

                Some(AgentEvent::MessageChunk(text))
            }
            "tool_call" => {
                // For tool calls, return the entire params as JSON string
                Some(AgentEvent::ToolCall(params_clone.to_string()))
            }
            _ => {
                // Ignore other event types (tool_call_update, plan, etc.)
                None
            }
        }
    }
}

#[async_trait]
impl Agent for QwenAdapter {
    async fn check_availability(&self) -> bool {
        self.resolve_qwen_command().is_ok()
    }

    async fn execute(
        &self,
        context: &ExecutionContext,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<AgentEvent, AgentError>> + Send>>, AgentError>
    {
        // 1. Ensure QWEN.md exists
        self.ensure_qwen_md(&context.project_path).await?;

        // 2. Resolve qwen command
        let cmd = self.resolve_qwen_command()?;

        // 3. Spawn qwen process with --experimental-acp flag
        let mut child = Command::new(&cmd)
            .arg("--experimental-acp")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .current_dir(&context.project_path)
            .spawn()
            .map_err(|e| {
                AgentError::ExecutionError(format!("Failed to spawn qwen process: {}", e))
            })?;

        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| AgentError::ExecutionError("Failed to capture stdin".to_string()))?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| AgentError::ExecutionError("Failed to capture stdout".to_string()))?;

        // 4. Send initialize request
        let init_req = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "clientCapabilities": {
                    "fs": {"readTextFile": false, "writeTextFile": false}
                },
                "protocolVersion": 1
            }
        });
        Self::send_request(&mut stdin, &init_req).await?;

        // 5. Send session/prompt request
        let prompt_req = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "session/prompt",
            "params": {
                "prompt": [{
                    "type": "text",
                    "text": context.instruction.clone()
                }]
            }
        });
        Self::send_request(&mut stdin, &prompt_req).await?;

        // 6. Create and return notification stream
        let stream = Self::create_notification_stream(stdout);
        Ok(Box::pin(stream))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qwen_adapter_new() {
        let adapter = QwenAdapter::new(
            "test-qwen".to_string(),
            "qwen-coder".to_string(),
            "You are a helpful assistant.".to_string(),
        );
        assert!(adapter.is_ok());

        let adapter = adapter.unwrap();
        assert_eq!(adapter.model, "qwen-coder");
        assert_eq!(adapter.system_prompt, "You are a helpful assistant.");
    }

    #[test]
    fn test_resolve_qwen_command() {
        let adapter = QwenAdapter::new(
            "test".to_string(),
            "qwen-coder".to_string(),
            "prompt".to_string(),
        )
        .unwrap();

        // Result depends on environment, just ensure it doesn't panic
        let _ = adapter.resolve_qwen_command();
    }

    #[tokio::test]
    async fn test_check_availability() {
        let adapter = QwenAdapter::new(
            "test".to_string(),
            "qwen-coder".to_string(),
            "prompt".to_string(),
        )
        .unwrap();

        // Will return false unless qwen CLI is installed
        // Don't assert the result, just ensure it doesn't panic
        let _ = adapter.check_availability().await;
    }

    #[tokio::test]
    async fn test_ensure_qwen_md() {
        let adapter = QwenAdapter::new(
            "test".to_string(),
            "qwen-coder".to_string(),
            "You are a test assistant.".to_string(),
        )
        .unwrap();

        let temp_dir = tempfile::tempdir().unwrap();
        let project_path = temp_dir.path().to_str().unwrap();

        // First call should create the file
        let result = adapter.ensure_qwen_md(project_path).await;
        assert!(result.is_ok());

        // Verify file was created
        let qwen_md_path = temp_dir.path().join("QWEN.md");
        assert!(qwen_md_path.exists());

        let content = tokio::fs::read_to_string(&qwen_md_path).await.unwrap();
        assert!(content.contains("# QWEN"));
        assert!(content.contains("You are a test assistant."));

        // Second call should be idempotent (no error)
        let result2 = adapter.ensure_qwen_md(project_path).await;
        assert!(result2.is_ok());
    }

    #[test]
    fn test_notification_deserialization() {
        let json = r#"{
            "method": "session/update",
            "params": {
                "sessionUpdate": "agent_message_chunk",
                "text": "Hello"
            }
        }"#;

        let notification: Result<Notification, _> = serde_json::from_str(json);
        assert!(notification.is_ok());

        let notification = notification.unwrap();
        assert_eq!(notification.method, "session/update");
    }

    #[test]
    fn test_session_update_deserialization() {
        let json = r#"{
            "sessionUpdate": "agent_message_chunk",
            "text": "Hello, world!"
        }"#;

        let update: Result<SessionUpdate, _> = serde_json::from_str(json);
        assert!(update.is_ok());

        let update = update.unwrap();
        assert_eq!(update.kind, Some("agent_message_chunk".to_string()));
        assert_eq!(update.text, Some("Hello, world!".to_string()));
    }

    #[test]
    fn test_session_update_with_type_field() {
        let json = r#"{
            "type": "tool_call",
            "content": {"name": "write"}
        }"#;

        let update: Result<SessionUpdate, _> = serde_json::from_str(json);
        assert!(update.is_ok());

        let update = update.unwrap();
        assert_eq!(update.type_field, Some("tool_call".to_string()));
        assert!(update.content.is_some());
    }

    #[test]
    fn test_convert_update_message_chunk() {
        let params = serde_json::json!({
            "sessionUpdate": "agent_message_chunk",
            "text": "Hello from Qwen!"
        });

        let event = QwenAdapter::convert_update(params);
        assert!(event.is_some());

        match event.unwrap() {
            AgentEvent::MessageChunk(text) => assert_eq!(text, "Hello from Qwen!"),
            _ => panic!("Expected MessageChunk"),
        }
    }

    #[test]
    fn test_convert_update_thought_chunk() {
        let params = serde_json::json!({
            "type": "agent_thought_chunk",
            "text": "Thinking..."
        });

        let event = QwenAdapter::convert_update(params);
        assert!(event.is_some());

        match event.unwrap() {
            AgentEvent::MessageChunk(text) => assert_eq!(text, "Thinking..."),
            _ => panic!("Expected MessageChunk"),
        }
    }

    #[test]
    fn test_convert_update_tool_call() {
        let params = serde_json::json!({
            "type": "tool_call",
            "name": "write",
            "input": {"path": "test.txt", "content": "Hello"}
        });

        let event = QwenAdapter::convert_update(params.clone());
        assert!(event.is_some());

        match event.unwrap() {
            AgentEvent::ToolCall(json) => {
                assert!(json.contains("tool_call"));
            }
            _ => panic!("Expected ToolCall"),
        }
    }

    #[test]
    fn test_convert_update_unknown_type() {
        let params = serde_json::json!({
            "type": "unknown_event",
            "data": "something"
        });

        let event = QwenAdapter::convert_update(params);
        assert!(event.is_none());
    }

    #[tokio::test]
    #[cfg(feature = "integration-tests")]
    async fn test_qwen_adapter_execute_integration() {
        use tokio_stream::StreamExt;

        let adapter = QwenAdapter::new(
            "test-qwen".to_string(),
            "qwen-coder".to_string(),
            "You are a helpful coding assistant.".to_string(),
        )
        .unwrap();

        // Skip test if qwen CLI is not installed
        if !adapter.check_availability().await {
            eprintln!("[test] Skipping integration test: qwen CLI not installed");
            return;
        }

        let temp_dir = tempfile::tempdir().unwrap();
        let project_path = temp_dir.path().to_str().unwrap();

        let context = ExecutionContext::new("Say hello in one word".to_string())
            .with_project_path(project_path.to_string());

        let mut stream = adapter.execute(&context).await.unwrap();
        let mut events = Vec::new();

        // Collect events with timeout to avoid hanging
        let timeout_duration = std::time::Duration::from_secs(30);
        let start = std::time::Instant::now();

        while let Some(result) = stream.next().await {
            if start.elapsed() > timeout_duration {
                eprintln!("[test] Timeout after 30 seconds");
                break;
            }

            match result {
                Ok(event) => {
                    eprintln!("[test] Event: {:?}", event);
                    let is_completed = matches!(event, AgentEvent::Completed);
                    events.push(event);
                    if is_completed {
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("[test] Error: {}", e);
                    break;
                }
            }
        }

        // Verify that we got at least a Completed event
        assert!(!events.is_empty(), "Should receive at least one event");
        assert!(
            events.iter().any(|e| matches!(e, AgentEvent::Completed)),
            "Should receive a Completed event"
        );

        // Verify QWEN.md was created
        let qwen_md_path = temp_dir.path().join("QWEN.md");
        assert!(qwen_md_path.exists(), "QWEN.md should be created");
    }
}
