//! Integration tests for CliExecutor.

use pk_core::agents::cli_executor::CliExecutor;
use tokio_stream::StreamExt;

#[tokio::test]
async fn test_cli_executor_execute_success() {
    // Get the path to the mock CLI script
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let mock_cli_path = format!("{}/tests/common/mock_cli.py", manifest_dir);

    // Execute the mock CLI
    let stream = CliExecutor::execute(
        "python3".to_string(),
        vec![mock_cli_path],
        manifest_dir.to_string(),
    );

    // Collect all JSON values from the stream
    let values: Vec<_> = stream
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .expect("Stream should not contain errors");

    // Verify we got 5 events
    assert_eq!(values.len(), 5, "Expected 5 JSON events from mock CLI");

    // Verify first event (system)
    assert_eq!(
        values[0].get("type").and_then(|v| v.as_str()),
        Some("system")
    );
    assert_eq!(
        values[0].get("message").and_then(|v| v.as_str()),
        Some("Starting mock CLI")
    );

    // Verify second event (assistant)
    assert_eq!(
        values[1].get("type").and_then(|v| v.as_str()),
        Some("assistant")
    );
    let content = values[1].get("content").and_then(|v| v.as_array());
    assert!(content.is_some());
    assert_eq!(content.unwrap().len(), 1);

    // Verify third event (tool_call)
    assert_eq!(
        values[2].get("type").and_then(|v| v.as_str()),
        Some("tool_call")
    );
    assert_eq!(values[2].get("name").and_then(|v| v.as_str()), Some("Read"));

    // Verify fourth event (assistant)
    assert_eq!(
        values[3].get("type").and_then(|v| v.as_str()),
        Some("assistant")
    );

    // Verify fifth event (result)
    assert_eq!(
        values[4].get("type").and_then(|v| v.as_str()),
        Some("result")
    );
    assert_eq!(
        values[4].get("session_id").and_then(|v| v.as_str()),
        Some("mock-session-123")
    );
}

#[tokio::test]
async fn test_cli_executor_invalid_json() {
    // Create a temporary script that outputs invalid JSON
    let temp_dir = tempfile::tempdir().unwrap();
    let script_path = temp_dir.path().join("bad_cli.py");
    std::fs::write(
        &script_path,
        r#"#!/usr/bin/env python3
print("not json")
print("{\"valid\": true}")
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

    let results: Vec<_> = stream.collect::<Vec<_>>().await;

    // First line should be an error (invalid JSON)
    assert!(results[0].is_err());

    // Second line should be valid
    assert!(results[1].is_ok());
}

#[tokio::test]
async fn test_cli_executor_empty_lines() {
    // Create a script with empty lines
    let temp_dir = tempfile::tempdir().unwrap();
    let script_path = temp_dir.path().join("empty_lines.py");
    std::fs::write(
        &script_path,
        r#"#!/usr/bin/env python3
import json

print()
print(json.dumps({"type": "test"}))
print()
print(json.dumps({"type": "test2"}))
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
        .expect("Stream should not contain errors");

    // Empty lines should be filtered out
    assert_eq!(values.len(), 2);
    assert_eq!(values[0].get("type").and_then(|v| v.as_str()), Some("test"));
    assert_eq!(
        values[1].get("type").and_then(|v| v.as_str()),
        Some("test2")
    );
}

#[tokio::test]
async fn test_cli_executor_command_not_found() {
    let stream = CliExecutor::execute(
        "nonexistent-command-xyz123".to_string(),
        vec![],
        ".".to_string(),
    );

    // Collect results - should get an error
    let results: Vec<_> = stream.collect::<Vec<_>>().await;

    // Should have at least one error
    assert!(!results.is_empty());
    assert!(results[0].is_err());
}
