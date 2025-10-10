//! Claude adapter implementation using Claude CLI subprocess.

use crate::agents::base::{Agent, AgentError, AgentEvent, ExecutionContext};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::pin::Pin;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio_stream::Stream;
use tokio_stream::StreamExt;

/// Claude adapter for executing instructions using Claude CLI.
///
/// This adapter spawns the `claude` CLI as a subprocess and parses its
/// JSON Lines output to create a stream of AgentEvents.
pub struct ClaudeAdapter {
    name: String,
    model: String,
    system_prompt: String,
    /// Session mapping: project_id -> session_id
    session_mapping: Arc<Mutex<HashMap<String, String>>>,
}

impl ClaudeAdapter {
    /// Create a new Claude adapter.
    ///
    /// # Arguments
    ///
    /// * `name` - The agent name from configuration
    /// * `model` - The Claude model to use (e.g., "claude-sonnet-4.5")
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
    ///
    /// Uses the last component of the path as the project ID.
    fn extract_project_id(project_path: &str) -> String {
        std::path::Path::new(project_path)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(project_path)
            .to_string()
    }

    /// Create a temporary settings file for the Claude CLI.
    ///
    /// The settings file contains the system prompt.
    fn create_settings_file(&self) -> Result<tempfile::NamedTempFile, AgentError> {
        use std::io::Write;

        let settings = serde_json::json!({
            "customSystemPrompt": self.system_prompt
        });

        let mut temp_file = tempfile::NamedTempFile::new()
            .map_err(|e| AgentError::ExecutionError(format!("Failed to create temp file: {}", e)))?;

        serde_json::to_writer(&mut temp_file, &settings)
            .map_err(|e| AgentError::ExecutionError(format!("Failed to write settings: {}", e)))?;

        // Flush to ensure data is written
        temp_file.flush()
            .map_err(|e| AgentError::ExecutionError(format!("Failed to flush settings: {}", e)))?;

        Ok(temp_file)
    }
}

#[async_trait]
impl Agent for ClaudeAdapter {
    async fn check_availability(&self) -> bool {
        // Check if claude CLI is installed by running "claude -h"
        match Command::new("claude")
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
        // 1. Create settings file
        let settings_file = self.create_settings_file()?;
        let settings_path = settings_file.path().to_str()
            .ok_or_else(|| AgentError::ExecutionError("Invalid settings path".to_string()))?
            .to_string();

        // 2. Get session ID for this project
        let project_id = Self::extract_project_id(&context.project_path);
        let session_id = {
            let mapping = self.session_mapping.lock().unwrap();
            mapping.get(&project_id).cloned()
        };

        // 3. Build command
        let mut cmd = Command::new("claude");
        cmd.arg("--settings").arg(&settings_path);
        cmd.arg("--model").arg(&self.model);
        cmd.arg("--permission-mode").arg("bypassPermissions");
        cmd.arg("--continue-conversation");

        // Tool filtering based on is_initial_prompt
        if context.is_initial_prompt {
            // Initial prompt: exclude TodoWrite
            cmd.arg("--allowed-tools").arg("Read,Write,Edit,MultiEdit,Bash,Glob,Grep,LS,WebFetch,WebSearch");
            cmd.arg("--disallowed-tools").arg("TodoWrite");
        } else {
            // Subsequent prompts: include TodoWrite
            cmd.arg("--allowed-tools").arg("Read,Write,Edit,MultiEdit,Bash,Glob,Grep,LS,WebFetch,WebSearch,TodoWrite");
        }

        // Session resumption
        if let Some(sid) = &session_id {
            cmd.arg("--resume-session-id").arg(sid);
        }

        // Prompt
        cmd.arg("--prompt").arg(&context.instruction);

        // Set working directory
        cmd.current_dir(&context.project_path);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        // 4. Spawn the process
        let mut child = cmd.spawn()
            .map_err(|e| AgentError::ExecutionError(format!("Failed to spawn claude CLI: {}", e)))?;

        let stdout = child.stdout.take()
            .ok_or_else(|| AgentError::ExecutionError("Failed to capture stdout".to_string()))?;

        // 5. Create stream of lines
        let reader = BufReader::new(stdout);
        let lines = reader.lines();
        let lines_stream = tokio_stream::wrappers::LinesStream::new(lines);

        // 6. Parse JSON Lines and convert to AgentEvents
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

                        // Parse JSON
                        match serde_json::from_str::<ClaudeMessage>(&line) {
                            Ok(msg) => {
                                convert_claude_message(msg, session_mapping, project_id).await
                            }
                            Err(e) => {
                                Some(Err(AgentError::StreamParseError(format!(
                                    "Failed to parse JSON: {} (line: {})",
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

/// Claude CLI message types (JSON Lines output).
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum ClaudeMessage {
    #[serde(rename = "system")]
    System {
        session_id: Option<String>,
        model: Option<String>,
    },
    #[serde(rename = "assistant")]
    Assistant {
        content: Vec<ContentBlock>,
    },
    #[serde(rename = "user")]
    User {
        content: String,
    },
    #[serde(rename = "result")]
    Result {
        session_id: Option<String>,
        duration_ms: Option<u64>,
        total_cost_usd: Option<f64>,
        num_turns: Option<u32>,
        is_error: Option<bool>,
    },
}

/// Content blocks within an assistant message.
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
enum ContentBlock {
    #[serde(rename = "text")]
    Text {
        text: String,
    },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    #[serde(rename = "tool_result")]
    ToolResult {
        tool_use_id: String,
        content: String,
    },
}

/// Convert Claude CLI message to AgentEvent.
async fn convert_claude_message(
    msg: ClaudeMessage,
    session_mapping: Arc<Mutex<HashMap<String, String>>>,
    project_id: String,
) -> Option<Result<AgentEvent, AgentError>> {
    match msg {
        ClaudeMessage::System { session_id, .. } => {
            // Save session ID
            if let Some(sid) = session_id {
                let mut mapping = session_mapping.lock().unwrap();
                mapping.insert(project_id, sid);
            }
            // System messages are not shown to UI
            None
        }
        ClaudeMessage::Assistant { content } => {
            // Process content blocks
            for block in content {
                match block {
                    ContentBlock::Text { text } => {
                        if !text.trim().is_empty() {
                            return Some(Ok(AgentEvent::MessageChunk(text)));
                        }
                    }
                    ContentBlock::ToolUse { name, input, .. } => {
                        let tool_json = serde_json::to_string(&serde_json::json!({
                            "name": name,
                            "input": input
                        })).unwrap_or_else(|_| format!("{{\"name\":\"{}\"}}", name));
                        return Some(Ok(AgentEvent::ToolCall(tool_json)));
                    }
                    ContentBlock::ToolResult { .. } => {
                        // Tool results are not shown to UI
                    }
                }
            }
            None
        }
        ClaudeMessage::User { .. } => {
            // User messages (echoes) are not shown
            None
        }
        ClaudeMessage::Result { session_id, .. } => {
            // Save session ID if present
            if let Some(sid) = session_id {
                let mut mapping = session_mapping.lock().unwrap();
                mapping.insert(project_id, sid);
            }
            // Signal completion
            Some(Ok(AgentEvent::Completed))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claude_adapter_new() {
        let adapter = ClaudeAdapter::new(
            "test".to_string(),
            "claude-sonnet-4.5".to_string(),
            "test prompt".to_string(),
        );
        assert!(adapter.is_ok());
    }

    #[test]
    fn test_extract_project_id() {
        assert_eq!(ClaudeAdapter::extract_project_id("/path/to/project"), "project");
        assert_eq!(ClaudeAdapter::extract_project_id("/tmp/test"), "test");
        assert_eq!(ClaudeAdapter::extract_project_id("relative/path"), "path");
    }

    #[test]
    fn test_create_settings_file() {
        let adapter = ClaudeAdapter::new(
            "test".to_string(),
            "claude-sonnet-4.5".to_string(),
            "test prompt".to_string(),
        ).unwrap();

        let settings_file = adapter.create_settings_file();
        assert!(settings_file.is_ok());

        let file = settings_file.unwrap();
        let path = file.path();
        assert!(path.exists());

        // Read and verify content
        let content = std::fs::read_to_string(path).unwrap();
        assert!(content.contains("customSystemPrompt"));
        assert!(content.contains("test prompt"));
    }

    #[tokio::test]
    async fn test_check_availability() {
        let adapter = ClaudeAdapter::new(
            "test".to_string(),
            "claude-sonnet-4.5".to_string(),
            "test prompt".to_string(),
        ).unwrap();

        // This will return false unless claude CLI is actually installed
        let available = adapter.check_availability().await;
        // We can't assert true/false as it depends on the environment
        // Just verify it doesn't panic
        let _ = available;
    }
}
