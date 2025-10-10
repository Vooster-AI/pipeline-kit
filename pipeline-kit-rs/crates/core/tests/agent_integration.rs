//! Integration tests for agent adapters.
//!
//! These tests verify the real CLI integration with actual agent CLIs.
//! Run with: `cargo test --features e2e-cli-tests`
//!
//! Prerequisites:
//! - Claude CLI: `claude` must be installed and configured
//! - Cursor CLI: `cursor-agent` must be installed
//! - Gemini CLI: `gemini-cli` must be installed and `GEMINI_API_KEY` set
//! - Codex CLI: `codex` must be installed and `OPENAI_API_KEY` set

use pk_core::agents::{AgentFactory, ExecutionContext};
use pk_protocol::agent_models;
use tokio_stream::StreamExt;

/// Helper to create test agent config.
fn create_agent_config(name: &str, model: &str, system_prompt: &str) -> agent_models::Agent {
    agent_models::Agent {
        name: name.to_string(),
        model: model.to_string(),
        description: format!("Test agent: {}", name),
        color: "blue".to_string(),
        system_prompt: system_prompt.to_string(),
    }
}

/// Helper to create test execution context.
fn create_test_context(instruction: &str, project_path: &str) -> ExecutionContext {
    ExecutionContext::new(instruction.to_string())
        .with_project_path(project_path.to_string())
        .with_initial_prompt(true)
}

// =============================================================================
// Feature-gated integration tests (require actual CLI installations)
// =============================================================================

#[cfg(feature = "e2e-cli-tests")]
mod real_cli_tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Run explicitly with --ignored flag
    async fn test_claude_adapter_real_execution() {
        // Check if Claude CLI is available
        let config = create_agent_config(
            "test-claude",
            "claude-sonnet-4.5",
            "You are a helpful assistant. Be concise.",
        );

        let agent = AgentFactory::create(&config).expect("Failed to create Claude agent");

        if !agent.check_availability().await {
            eprintln!("Skipping test: Claude CLI not available");
            return;
        }

        // Create temporary test directory
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let project_path = temp_dir.path().to_str().unwrap();

        let context = create_test_context("Say 'Hello' in one word", project_path);

        // Execute and collect events
        let stream = agent
            .execute(&context)
            .await
            .expect("Failed to execute agent");

        let events: Vec<_> = stream.collect().await;

        // Verify we got some events
        assert!(!events.is_empty(), "Should receive at least one event");

        // Check for successful completion
        let has_completion = events
            .iter()
            .any(|e| matches!(e, Ok(pk_core::agents::AgentEvent::Completed)));
        assert!(has_completion, "Should complete successfully");

        println!("Claude execution test passed with {} events", events.len());
    }

    #[tokio::test]
    #[ignore]
    async fn test_cursor_adapter_real_execution() {
        let config = create_agent_config(
            "test-cursor",
            "gpt-5",
            "You are a helpful coding assistant.",
        );

        let agent = AgentFactory::create(&config).expect("Failed to create Cursor agent");

        if !agent.check_availability().await {
            eprintln!("Skipping test: Cursor CLI not available");
            return;
        }

        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let project_path = temp_dir.path().to_str().unwrap();

        let context = create_test_context("What is 2+2? Answer with just the number.", project_path);

        let stream = agent
            .execute(&context)
            .await
            .expect("Failed to execute agent");

        let events: Vec<_> = stream.collect().await;

        assert!(!events.is_empty(), "Should receive at least one event");

        // Verify AGENTS.md was created
        let agents_md_path = temp_dir.path().join("AGENTS.md");
        assert!(
            agents_md_path.exists(),
            "AGENTS.md should be created by CursorAdapter"
        );

        println!("Cursor execution test passed with {} events", events.len());
    }

    #[tokio::test]
    #[ignore]
    async fn test_gemini_adapter_real_execution() {
        let config = create_agent_config(
            "test-gemini",
            "gemini-2.5-pro",
            "You are a helpful assistant.",
        );

        let agent = AgentFactory::create(&config).expect("Failed to create Gemini agent");

        if !agent.check_availability().await {
            eprintln!("Skipping test: Gemini CLI or API key not available");
            return;
        }

        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let project_path = temp_dir.path().to_str().unwrap();

        let context = create_test_context("What is the capital of France? One word answer.", project_path);

        let stream = agent
            .execute(&context)
            .await
            .expect("Failed to execute agent");

        let events: Vec<_> = stream.collect().await;

        assert!(!events.is_empty(), "Should receive at least one event");

        let has_message = events
            .iter()
            .any(|e| matches!(e, Ok(pk_core::agents::AgentEvent::MessageChunk(_))));
        assert!(has_message, "Should receive at least one message chunk");

        println!("Gemini execution test passed with {} events", events.len());
    }

    #[tokio::test]
    #[ignore]
    async fn test_codex_adapter_real_execution() {
        let config = create_agent_config(
            "test-codex",
            "codex",
            "You are a helpful coding assistant.",
        );

        let agent = AgentFactory::create(&config).expect("Failed to create Codex agent");

        if !agent.check_availability().await {
            eprintln!("Skipping test: Codex CLI or API key not available");
            return;
        }

        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let project_path = temp_dir.path().to_str().unwrap();

        let context = create_test_context("Write a hello world comment in Python", project_path);

        let stream = agent
            .execute(&context)
            .await
            .expect("Failed to execute agent");

        let events: Vec<_> = stream.collect().await;

        assert!(!events.is_empty(), "Should receive at least one event");

        // Verify AGENTS.md was created
        let agents_md_path = temp_dir.path().join("AGENTS.md");
        assert!(
            agents_md_path.exists(),
            "AGENTS.md should be created by CodexAdapter"
        );

        // Verify rollout directory was created
        let rollout_dir = temp_dir
            .path()
            .join(".pipeline-kit")
            .join("codex_rollouts");
        assert!(
            rollout_dir.exists(),
            "Rollout directory should be created by CodexAdapter"
        );

        println!("Codex execution test passed with {} events", events.len());
    }

    #[tokio::test]
    #[ignore]
    async fn test_session_persistence() {
        // Test that sessions are properly managed across multiple executions
        let config = create_agent_config(
            "test-session",
            "claude-sonnet-4.5",
            "You are a helpful assistant.",
        );

        let agent = AgentFactory::create(&config).expect("Failed to create agent");

        if !agent.check_availability().await {
            eprintln!("Skipping test: Claude CLI not available");
            return;
        }

        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let project_path = temp_dir.path().to_str().unwrap();

        // First execution
        let context1 = create_test_context("Remember this number: 42", project_path);
        let stream1 = agent.execute(&context1).await.expect("Failed to execute");
        let _events1: Vec<_> = stream1.collect().await;

        // Second execution (should resume session)
        let context2 = ExecutionContext::new("What number did I tell you to remember?".to_string())
            .with_project_path(project_path.to_string())
            .with_initial_prompt(false); // Not initial prompt

        let stream2 = agent.execute(&context2).await.expect("Failed to execute");
        let events2: Vec<_> = stream2.collect().await;

        assert!(!events2.is_empty(), "Should receive events from resumed session");

        println!(
            "Session persistence test passed with {} events",
            events2.len()
        );
    }
}

// =============================================================================
// Mock-based integration tests (always run, no CLI required)
// =============================================================================

#[tokio::test]
async fn test_agent_factory_integration() {
    // Test that factory correctly creates different agent types
    let configs = vec![
        create_agent_config("claude", "claude-sonnet-4.5", "Test"),
        create_agent_config("cursor", "gpt-5", "Test"),
        create_agent_config("gemini", "gemini-2.5-pro", "Test"),
        create_agent_config("codex", "codex", "Test"),
        create_agent_config("mock", "unknown-model", "Test"),
    ];

    for config in configs {
        let agent = AgentFactory::create(&config);
        assert!(
            agent.is_ok(),
            "Factory should create agent for model: {}",
            config.model
        );
    }
}

#[tokio::test]
async fn test_execution_context_integration() {
    // Test that execution context is properly constructed
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let project_path = temp_dir.path().to_str().unwrap();

    let context = ExecutionContext::new("test instruction".to_string())
        .with_project_path(project_path.to_string())
        .with_initial_prompt(true);

    assert_eq!(context.instruction, "test instruction");
    assert_eq!(context.project_path, project_path);
    assert!(context.is_initial_prompt);
    assert!(context.attachments.is_empty());
}

#[tokio::test]
async fn test_mock_agent_integration() {
    // Test mock agent for development/testing
    let config = create_agent_config("mock", "test-model", "Test system prompt");
    let agent = AgentFactory::create(&config).expect("Failed to create mock agent");

    assert!(
        agent.check_availability().await,
        "Mock agent should always be available"
    );

    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let project_path = temp_dir.path().to_str().unwrap();
    let context = create_test_context("Test instruction", project_path);

    let stream = agent
        .execute(&context)
        .await
        .expect("Failed to execute mock agent");

    let events: Vec<_> = stream.collect().await;

    assert!(!events.is_empty(), "Mock agent should produce events");

    // Mock agent should complete successfully
    let has_completion = events
        .iter()
        .any(|e| matches!(e, Ok(pk_core::agents::AgentEvent::Completed)));
    assert!(has_completion, "Mock agent should complete");
}
