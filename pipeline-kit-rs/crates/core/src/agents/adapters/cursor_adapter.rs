//! Cursor adapter implementation using Cursor Agent CLI subprocess.

use crate::agents::base::{Agent, AgentError, AgentEvent, ExecutionContext};
use async_trait::async_trait;
use serde::Deserialize;
use std::collections::HashMap;
use std::pin::Pin;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use tokio::fs;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio_stream::Stream;
use tokio_stream::StreamExt;

/// Cursor adapter for executing instructions using Cursor Agent CLI.
///
/// This adapter spawns the `cursor-agent` CLI as a subprocess and parses its
/// NDJSON (Newline-Delimited JSON) output to create a stream of AgentEvents.
pub struct CursorAdapter {
    name: String,
    model: String,
    system_prompt: String,
    /// Session mapping: project_id -> session_id
    session_mapping: Arc<Mutex<HashMap<String, String>>>,
}

impl CursorAdapter {
    /// Create a new Cursor adapter.
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
    /// Cursor uses AGENTS.md for system prompts.
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
}

#[async_trait]
impl Agent for CursorAdapter {
    async fn check_availability(&self) -> bool {
        // Check if cursor-agent CLI is installed
        match Command::new("cursor-agent")
            .arg("-h")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await
        {
            Ok(status) => status.success(),
            Err(_) => false,
        }
    }

    async fn execute(
        &self,
        context: &ExecutionContext,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<AgentEvent, AgentError>> + Send>>, AgentError> {
        // 1. Ensure AGENTS.md exists
        self.ensure_agent_md(&context.project_path).await?;

        // 2. Get session ID
        let project_id = Self::extract_project_id(&context.project_path);
        let session_id = {
            let mapping = self.session_mapping.lock().unwrap();
            mapping.get(&project_id).cloned()
        };

        // 3. Build command
        let mut cmd = Command::new("cursor-agent");
        cmd.arg("--force");
        cmd.arg("-p").arg(&context.instruction);
        cmd.arg("--output-format").arg("stream-json");
        cmd.arg("-m").arg(&self.model);

        // Session resumption
        if let Some(sid) = &session_id {
            cmd.arg("--resume").arg(sid);
        }

        // API key from environment
        if let Ok(api_key) = std::env::var("CURSOR_API_KEY") {
            cmd.arg("--api-key").arg(&api_key);
        }

        // Set working directory
        cmd.current_dir(&context.project_path);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        // 4. Spawn process
        let mut child = cmd.spawn()
            .map_err(|e| AgentError::ExecutionError(format!("Failed to spawn cursor-agent: {}", e)))?;

        let stdout = child.stdout.take()
            .ok_or_else(|| AgentError::ExecutionError("Failed to capture stdout".to_string()))?;

        // 5. Create NDJSON stream
        let reader = BufReader::new(stdout);
        let lines = reader.lines();
        let lines_stream = tokio_stream::wrappers::LinesStream::new(lines);

        // 6. Parse and convert events
        let session_mapping = self.session_mapping.clone();
        let project_id_clone = project_id.clone();

        let events_stream = lines_stream.filter_map(move |line_result| {
            let session_mapping = session_mapping.clone();
            let project_id = project_id_clone.clone();

            async move {
                match line_result {
                    Ok(line) => {
                        if line.trim().is_empty() {
                            return None;
                        }

                        match serde_json::from_str::<CursorEvent>(&line) {
                            Ok(event) => {
                                convert_cursor_event(event, session_mapping, project_id).await
                            }
                            Err(e) => {
                                Some(Err(AgentError::StreamParseError(format!(
                                    "Failed to parse NDJSON: {} (line: {})",
                                    e, line
                                ))))
                            }
                        }
                    }
                    Err(e) => Some(Err(AgentError::StreamParseError(e.to_string()))),
                }
            }
        });

        Ok(Box::pin(events_stream))
    }
}

/// Cursor NDJSON event structure.
#[derive(Debug, Deserialize)]
struct CursorEvent {
    #[serde(rename = "type")]
    event_type: String,
    subtype: Option<String>,
    message: Option<serde_json::Value>,
    tool_call: Option<serde_json::Value>,
    session_id: Option<String>,
    duration_ms: Option<u64>,
}

/// Convert Cursor event to AgentEvent.
async fn convert_cursor_event(
    event: CursorEvent,
    session_mapping: Arc<Mutex<HashMap<String, String>>>,
    project_id: String,
) -> Option<Result<AgentEvent, AgentError>> {
    match event.event_type.as_str() {
        "system" => {
            // System initialization (hidden from UI)
            None
        }
        "user" => {
            // Echo back (suppress)
            None
        }
        "assistant" => {
            // Text delta
            if let Some(msg) = event.message {
                if let Some(content_array) = msg.get("content").and_then(|c| c.as_array()) {
                    for item in content_array {
                        if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                            if !text.trim().is_empty() {
                                return Some(Ok(AgentEvent::MessageChunk(text.to_string())));
                            }
                        }
                    }
                }
            }
            None
        }
        "tool_call" => {
            if let Some(subtype) = &event.subtype {
                if subtype == "started" {
                    // Tool call started
                    if let Some(tool_call) = event.tool_call {
                        return Some(Ok(AgentEvent::ToolCall(tool_call.to_string())));
                    }
                }
                // "completed" subtype is not shown to UI
            }
            None
        }
        "result" => {
            // Save session ID
            if let Some(sid) = event.session_id {
                let mut mapping = session_mapping.lock().unwrap();
                mapping.insert(project_id, sid);
            }
            // Signal completion
            Some(Ok(AgentEvent::Completed))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_adapter_new() {
        let adapter = CursorAdapter::new(
            "test".to_string(),
            "gpt-5".to_string(),
            "test prompt".to_string(),
        );
        assert!(adapter.is_ok());
    }

    #[test]
    fn test_extract_project_id() {
        assert_eq!(CursorAdapter::extract_project_id("/path/to/project"), "project");
        assert_eq!(CursorAdapter::extract_project_id("/tmp/test"), "test");
    }

    #[tokio::test]
    async fn test_check_availability() {
        let adapter = CursorAdapter::new(
            "test".to_string(),
            "gpt-5".to_string(),
            "test prompt".to_string(),
        ).unwrap();

        // This will return false unless cursor-agent CLI is actually installed
        let available = adapter.check_availability().await;
        let _ = available;
    }

    #[tokio::test]
    async fn test_ensure_agent_md() {
        let adapter = CursorAdapter::new(
            "test".to_string(),
            "gpt-5".to_string(),
            "test system prompt".to_string(),
        ).unwrap();

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
}
