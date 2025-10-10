//! E2E tests for pipeline execution.
//!
//! These tests verify end-to-end pipeline execution flows including:
//! - Starting pipelines
//! - Sequential agent execution
//! - HUMAN_REVIEW pausing/resuming
//! - Process completion and failure
//! - Error handling and recovery

mod common;

use common::*;
use pk_core::agents::manager::AgentManager;
use pk_core::engine::PipelineEngine;
use pk_core::state::manager::StateManager;
use pk_protocol::ipc::Event;
use pk_protocol::process_models::ProcessStatus;
use std::time::Duration;
use tokio::sync::mpsc;

/// Helper function to collect events from a channel until timeout or completion.
async fn collect_events_until_timeout(
    rx: &mut mpsc::Receiver<Event>,
    timeout: Duration,
) -> Vec<Event> {
    let mut events = Vec::new();
    let start = tokio::time::Instant::now();

    while start.elapsed() < timeout {
        match tokio::time::timeout(Duration::from_millis(100), rx.recv()).await {
            Ok(Some(event)) => {
                let is_terminal = matches!(
                    &event,
                    Event::ProcessCompleted { .. } | Event::ProcessKilled { .. }
                );
                events.push(event);
                if is_terminal {
                    break;
                }
            }
            Ok(None) => break,  // Channel closed
            Err(_) => continue, // Timeout, keep waiting
        }
    }

    events
}

/// RED: Test simple pipeline execution with mock agents.
///
/// Acceptance criteria:
/// 1. Pipeline starts successfully
/// 2. Mock agents execute in sequence
/// 3. Events are emitted in correct order
/// 4. Process completes successfully
#[tokio::test]
async fn test_simple_pipeline_with_mock_agents() {
    // Given: A pipeline with 2 mock agents
    let agents = vec![create_test_agent("agent-1"), create_test_agent("agent-2")];
    let agent_manager = AgentManager::new(agents);

    let pipeline = create_simple_pipeline("test-pipeline", 2);

    // When: Execute the pipeline
    let (events_tx, mut events_rx) = mpsc::channel(100);
    let engine = PipelineEngine::new(agent_manager);

    // Create a process for the pipeline
    let process = create_test_process("test-pipeline", ProcessStatus::Pending);

    tokio::spawn(async move {
        let _ = engine.run(&pipeline, process, events_tx).await;
    });

    // Then: Collect events
    let events = collect_events_until_timeout(&mut events_rx, Duration::from_secs(5)).await;

    // Assertions
    println!("Received {} events", events.len());
    for (i, event) in events.iter().enumerate() {
        println!("Event {}: {:?}", i, event);
    }

    // RED: These assertions will fail until we implement proper integration
    assert!(
        !events.is_empty(),
        "Should receive events from pipeline execution"
    );

    assert!(
        assert_has_process_started(&events),
        "Should have ProcessStarted event"
    );
}

/// RED: Test pipeline with HUMAN_REVIEW step.
///
/// Acceptance criteria:
/// 1. Pipeline pauses at HUMAN_REVIEW
/// 2. Status changes to HumanReview
/// 3. Pipeline can be resumed
/// 4. Execution continues after resume
#[tokio::test]
async fn test_pipeline_pauses_at_human_review() {
    // Given: A pipeline with HUMAN_REVIEW
    let agents = vec![create_test_agent("agent-1"), create_test_agent("agent-2")];
    let agent_manager = AgentManager::new(agents);

    let pipeline = create_pipeline_with_human_review("test-pipeline");

    // When: Execute the pipeline
    let (events_tx, mut events_rx) = mpsc::channel(100);
    let engine = PipelineEngine::new(agent_manager);

    // Create a process for the pipeline
    let process = create_test_process("test-pipeline", ProcessStatus::Pending);

    tokio::spawn(async move {
        let _ = engine.run(&pipeline, process, events_tx).await;
    });

    // Then: Should pause at HUMAN_REVIEW
    let events = collect_events_until_timeout(&mut events_rx, Duration::from_secs(5)).await;

    println!("Received {} events", events.len());

    // RED: This will fail until we implement proper HUMAN_REVIEW handling
    assert!(
        assert_has_status_update(&events, ProcessStatus::HumanReview),
        "Should reach HumanReview status"
    );
}

/// RED: Test full pipeline execution lifecycle with StateManager.
///
/// Acceptance criteria:
/// 1. StateManager tracks process state
/// 2. Process can be started via Op::StartPipeline
/// 3. Process can be queried via StateManager
/// 4. Process completes and is marked as completed
#[tokio::test]
async fn test_full_pipeline_lifecycle_with_state_manager() {
    // Given: StateManager and AgentManager
    let agents = vec![create_test_agent("agent-1")];
    let agent_manager = AgentManager::new(agents);

    let (events_tx, mut _events_rx) = mpsc::channel(100);
    let state_manager = StateManager::new(agent_manager, events_tx.clone());

    // Load pipeline configuration
    let pipeline = create_simple_pipeline("test-pipeline", 1);

    // When: Start pipeline via StateManager
    // TODO: This requires StateManager to have a start_pipeline method
    // For now, we'll test the structure

    // RED: This test structure is ready but will fail until we implement
    // the proper integration between StateManager and PipelineEngine

    // Placeholder assertion to make test compile
    let _ = state_manager;
    let _ = pipeline;
    // TODO: Add proper assertions once StateManager integration is complete
}

/// RED: Test error handling when agent fails.
///
/// Acceptance criteria:
/// 1. Failed agent emits error event
/// 2. Process status changes to Failed
/// 3. Error message is logged
/// 4. Subsequent agents don't execute
#[tokio::test]
async fn test_pipeline_handles_agent_failure() {
    // This test is a placeholder for future implementation
    // We'll need MockFailureAgent to be properly integrated

    // TODO: Implement agent failure test once MockFailureAgent is integrated
}
