//! Integration tests for PipelineEngine.
//!
//! These tests verify that the PipelineEngine correctly:
//! - Executes pipeline steps sequentially
//! - Emits appropriate events through the channel
//! - Handles HUMAN_REVIEW steps by pausing execution
//! - Manages process state transitions

use pk_core::agents::manager::AgentManager;
use pk_core::engine::PipelineEngine;
use pk_protocol::agent_models::Agent as AgentConfig;
use pk_protocol::ipc::Event;
use pk_protocol::pipeline_models::MasterAgentConfig;
use pk_protocol::pipeline_models::Pipeline;
use pk_protocol::pipeline_models::ProcessStep;
use pk_protocol::process_models::ProcessStatus;
use std::collections::HashMap;
use tokio::sync::mpsc;

fn create_test_agent_config(name: &str) -> AgentConfig {
    AgentConfig {
        name: name.to_string(),
        description: format!("Test agent {}", name),
        model: "test-model".to_string(),
        color: "blue".to_string(),
        system_prompt: "Test prompt".to_string(),
    }
}

fn create_test_pipeline(name: &str, steps: Vec<ProcessStep>) -> Pipeline {
    Pipeline {
        name: name.to_string(),
        required_reference_file: HashMap::new(),
        output_file: HashMap::new(),
        master: MasterAgentConfig {
            model: "test-model".to_string(),
            system_prompt: "Test orchestration".to_string(),
            process: steps,
        },
        sub_agents: vec!["agent1".to_string(), "agent2".to_string()],
    }
}

/// RED: This test should fail because PipelineEngine is not implemented yet.
///
/// Acceptance criteria:
/// 1. ProcessStarted event is emitted when pipeline begins
/// 2. ProcessStatusUpdate(Running) is emitted when execution starts
/// 3. Each agent step executes sequentially
/// 4. ProcessStatusUpdate(HumanReview) is emitted when HUMAN_REVIEW is reached
/// 5. Execution pauses at HUMAN_REVIEW step
#[tokio::test]
async fn test_pipeline_engine_sequential_execution_with_human_review() {
    // Setup: Create a 2-agent pipeline with HUMAN_REVIEW in between
    let agent_configs = vec![
        create_test_agent_config("agent1"),
        create_test_agent_config("agent2"),
    ];

    let agent_manager = AgentManager::new(agent_configs);

    let steps = vec![
        ProcessStep::Agent("agent1".to_string()),
        ProcessStep::HumanReview(pk_protocol::pipeline_models::HumanReviewMarker),
        ProcessStep::Agent("agent2".to_string()),
    ];

    let pipeline = create_test_pipeline("test-pipeline", steps);

    // Create event channel
    let (events_tx, mut events_rx) = mpsc::channel(100);

    // Execute pipeline
    let engine = PipelineEngine::new(agent_manager);

    // Create initial process
    let process = pk_protocol::Process {
        id: uuid::Uuid::new_v4(),
        pipeline_name: pipeline.name.clone(),
        status: ProcessStatus::Pending,
        current_step_index: 0,
        logs: Vec::new(),
        started_at: chrono::Utc::now(),
        completed_at: None,
        resume_notifier: std::sync::Arc::new(tokio::sync::Notify::new()),
    };

    let handle = tokio::spawn(async move { engine.run(&pipeline, process, events_tx).await });

    // Collect events
    let mut received_events = Vec::new();

    // Timeout for receiving events (in case the implementation blocks)
    let timeout_duration = std::time::Duration::from_secs(2);

    while let Ok(Some(event)) = tokio::time::timeout(timeout_duration, events_rx.recv()).await {
        let should_break = matches!(
            &event,
            Event::ProcessStatusUpdate {
                status: ProcessStatus::HumanReview,
                ..
            }
        );
        received_events.push(event);

        // Stop collecting after we receive HumanReview status
        if should_break {
            break;
        }
    }

    // Verify events
    assert!(
        !received_events.is_empty(),
        "Should have received at least some events"
    );

    // 1. First event should be ProcessStarted
    assert!(
        matches!(
            &received_events[0],
            Event::ProcessStarted { pipeline_name, .. } if pipeline_name == "test-pipeline"
        ),
        "First event should be ProcessStarted"
    );

    // 2. Should have ProcessStatusUpdate(Running) at some point
    let has_running_status = received_events.iter().any(|e| {
        matches!(
            e,
            Event::ProcessStatusUpdate {
                status: ProcessStatus::Running,
                ..
            }
        )
    });
    assert!(has_running_status, "Should have Running status update");

    // 3. Should eventually reach HumanReview status
    let has_human_review = received_events.iter().any(|e| {
        matches!(
            e,
            Event::ProcessStatusUpdate {
                status: ProcessStatus::HumanReview,
                ..
            }
        )
    });
    assert!(has_human_review, "Should reach HumanReview status");

    // 4. Should have log chunks from agent1 execution
    let has_log_chunks = received_events
        .iter()
        .any(|e| matches!(e, Event::ProcessLogChunk { .. }));
    assert!(
        has_log_chunks,
        "Should have log chunks from agent execution"
    );

    // Clean up
    drop(events_rx);
    let _ = handle.await;
}

/// RED: Test that pipeline completes successfully without HUMAN_REVIEW
#[tokio::test]
async fn test_pipeline_engine_completes_without_human_review() {
    let agent_configs = vec![
        create_test_agent_config("agent1"),
        create_test_agent_config("agent2"),
    ];

    let agent_manager = AgentManager::new(agent_configs);

    let steps = vec![
        ProcessStep::Agent("agent1".to_string()),
        ProcessStep::Agent("agent2".to_string()),
    ];

    let pipeline = create_test_pipeline("simple-pipeline", steps);

    let (events_tx, mut events_rx) = mpsc::channel(100);

    let engine = PipelineEngine::new(agent_manager);

    // Create initial process
    let process = pk_protocol::Process {
        id: uuid::Uuid::new_v4(),
        pipeline_name: pipeline.name.clone(),
        status: ProcessStatus::Pending,
        current_step_index: 0,
        logs: Vec::new(),
        started_at: chrono::Utc::now(),
        completed_at: None,
        resume_notifier: std::sync::Arc::new(tokio::sync::Notify::new()),
    };

    let handle = tokio::spawn(async move { engine.run(&pipeline, process, events_tx).await });

    let mut received_events = Vec::new();
    let timeout_duration = std::time::Duration::from_secs(2);

    while let Ok(Some(event)) = tokio::time::timeout(timeout_duration, events_rx.recv()).await {
        let is_completed = matches!(&event, Event::ProcessCompleted { .. });
        received_events.push(event);

        // Stop after completion
        if is_completed {
            break;
        }
    }

    // Should have ProcessCompleted event
    let has_completed = received_events
        .iter()
        .any(|e| matches!(e, Event::ProcessCompleted { .. }));
    assert!(has_completed, "Pipeline should complete successfully");

    // Should NOT have HumanReview status
    let has_human_review = received_events.iter().any(|e| {
        matches!(
            e,
            Event::ProcessStatusUpdate {
                status: ProcessStatus::HumanReview,
                ..
            }
        )
    });
    assert!(!has_human_review, "Should not have HumanReview status");

    drop(events_rx);
    let _ = handle.await;
}
