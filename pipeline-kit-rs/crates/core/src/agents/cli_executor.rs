//! Common CLI subprocess executor for agent adapters.
//!
//! This module provides a unified interface for executing CLI-based agents
//! and parsing their JSON Lines / NDJSON output streams.

use crate::agents::base::AgentError;
use std::pin::Pin;
use std::process::Stdio;
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;
use tokio::process::Command;
use tokio_stream::Stream;

/// CLI executor for agent adapters.
///
/// This struct provides common functionality for spawning subprocess-based
/// CLI tools and parsing their JSON Lines output.
pub struct CliExecutor;

impl CliExecutor {
    /// Execute a CLI command and parse its stdout as JSON Lines/NDJSON.
    ///
    /// # Arguments
    ///
    /// * `command` - The command to execute (e.g., "python3", "claude")
    /// * `args` - Command line arguments
    /// * `working_dir` - Working directory for the command
    ///
    /// # Returns
    ///
    /// A stream of `serde_json::Value` objects, one per line of JSON output.
    /// Empty lines are automatically filtered out. Lines that fail to parse
    /// as JSON will yield `AgentError::StreamParseError`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use pk_core::agents::cli_executor::CliExecutor;
    /// use tokio_stream::StreamExt;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let stream = CliExecutor::execute(
    ///         "echo".to_string(),
    ///         vec![r#"{"type":"test"}"#.to_string()],
    ///         ".".to_string(),
    ///     );
    ///
    ///     let values: Vec<_> = stream.collect().await;
    ///     println!("Got {} values", values.len());
    /// }
    /// ```
    pub fn execute(
        command: String,
        args: Vec<String>,
        working_dir: String,
    ) -> Pin<Box<dyn Stream<Item = Result<serde_json::Value, AgentError>> + Send>> {
        // Create async stream using async_stream::stream macro
        let stream = async_stream::stream! {
            // Build and spawn the command
            let mut cmd = Command::new(&command);
            cmd.args(&args);
            cmd.current_dir(&working_dir);
            cmd.stdout(Stdio::piped());
            cmd.stderr(Stdio::piped());

            let mut child = match cmd.spawn() {
                Ok(child) => child,
                Err(e) => {
                    yield Err(AgentError::ExecutionError(format!(
                        "Failed to spawn command '{}': {}",
                        command, e
                    )));
                    return;
                }
            };

            // Capture stdout
            let stdout = match child.stdout.take() {
                Some(stdout) => stdout,
                None => {
                    yield Err(AgentError::ExecutionError(
                        "Failed to capture stdout".to_string()
                    ));
                    return;
                }
            };

            // Create buffered reader for line-by-line processing
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();

            // Read lines and parse as JSON
            while let Ok(Some(line)) = lines.next_line().await {
                // Skip empty lines
                if line.trim().is_empty() {
                    continue;
                }

                // Try to parse as JSON
                match serde_json::from_str::<serde_json::Value>(&line) {
                    Ok(value) => {
                        yield Ok(value);
                    }
                    Err(e) => {
                        yield Err(AgentError::StreamParseError(format!(
                            "Failed to parse JSON: {} (line: {})",
                            e, line
                        )));
                    }
                }
            }

            // Wait for the process to complete
            // We don't need to check exit status here, as any output errors
            // will have been captured above
            let _ = child.wait().await;
        };

        Box::pin(stream)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_stream::StreamExt;

    #[tokio::test]
    async fn test_execute_echo_json() {
        // Use echo to output a JSON line
        let stream = CliExecutor::execute(
            "echo".to_string(),
            vec![r#"{"type":"test","value":42}"#.to_string()],
            ".".to_string(),
        );

        let values: Vec<_> = stream
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .expect("Should parse JSON successfully");

        assert_eq!(values.len(), 1);
        assert_eq!(values[0].get("type").and_then(|v| v.as_str()), Some("test"));
        assert_eq!(values[0].get("value").and_then(|v| v.as_i64()), Some(42));
    }

    #[tokio::test]
    async fn test_execute_invalid_command() {
        let stream = CliExecutor::execute(
            "nonexistent-command-xyz".to_string(),
            vec![],
            ".".to_string(),
        );

        let results: Vec<_> = stream.collect::<Vec<_>>().await;

        assert!(!results.is_empty());
        assert!(results[0].is_err());

        if let Err(AgentError::ExecutionError(msg)) = &results[0] {
            assert!(msg.contains("Failed to spawn command"));
        } else {
            panic!("Expected ExecutionError");
        }
    }

    #[tokio::test]
    async fn test_execute_filters_empty_lines() {
        // Create a temporary script that outputs empty lines
        let temp_dir = tempfile::tempdir().unwrap();
        let script_path = temp_dir.path().join("test.py");
        std::fs::write(
            &script_path,
            r#"#!/usr/bin/env python3
import json
print()
print(json.dumps({"num": 1}))
print()
print(json.dumps({"num": 2}))
print()
"#,
        )
        .unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&script_path, std::fs::Permissions::from_mode(0o755)).unwrap();
        }

        let stream = CliExecutor::execute(
            "python3".to_string(),
            vec![script_path.to_str().unwrap().to_string()],
            temp_dir.path().to_str().unwrap().to_string(),
        );

        let values: Vec<_> = stream
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .expect("Should parse JSON successfully");

        // Should only get 2 values (empty lines filtered)
        assert_eq!(values.len(), 2);
        assert_eq!(values[0].get("num").and_then(|v| v.as_i64()), Some(1));
        assert_eq!(values[1].get("num").and_then(|v| v.as_i64()), Some(2));
    }
}
