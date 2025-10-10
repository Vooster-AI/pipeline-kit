//! E2E tests for pipeline execution.
//!
//! These tests verify end-to-end pipeline execution flows including:
//! - Starting pipelines
//! - Sequential agent execution
//! - HUMAN_REVIEW pausing/resuming
//! - Process completion and failure
//! - Error handling and recovery
//!
//! ## Test Priority (Phase 2.1-A: Core Happy Path)
//!
//! Priority 1 (Essential):
//! - test_single_agent_complete_execution (1.1.1)
//! - test_multi_agent_sequential_execution (1.1.2)
//! - test_human_review_pause_and_resume (1.2.1 + 1.2.2)
//! - test_agent_failure_stops_pipeline (2.1.1)
//! - test_event_sequence_validation (1.3.1)

mod common;

use common::assertions::*;
use common::fixtures::*;
use pk_core::agents::manager::AgentManager;
use pk_core::engine::PipelineEngine;
use pk_core::state::manager::StateManager;
use pk_protocol::ipc::Event;
use pk_protocol::pipeline_models::ProcessStep;
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
                    Event::ProcessCompleted { .. }
                        | Event::ProcessKilled { .. }
                        | Event::ProcessError { .. }
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

/// Priority 1.1.1: Single agent complete execution
///
/// Acceptance criteria:
/// 1. Process lifecycle: Pending → Running → Completed
/// 2. Events emitted: ProcessStarted → StatusUpdate(Running) → LogChunk → ProcessCompleted
/// 3. Logs are captured in process state
/// 4. Timestamps are set correctly
#[tokio::test]
async fn test_single_agent_complete_execution() {
    // Given: A pipeline with a single mock success agent
    let agents = vec![create_test_agent("agent-1")];
    let agent_manager = AgentManager::new(agents);

    let steps = vec![ProcessStep::Agent("agent-1".to_string())];
    let pipeline = create_test_pipeline("single-agent-pipeline", steps);

    // When: Execute the pipeline via PipelineEngine
    let (events_tx, mut events_rx) = mpsc::channel(100);
    let engine = PipelineEngine::new(agent_manager);

    let process = create_test_process("single-agent-pipeline", ProcessStatus::Pending);
    let process_id = process.id;

    tokio::spawn(async move {
        let _ = engine.run(&pipeline, process, events_tx).await;
    });

    // Then: Collect and verify events
    let events = collect_events_until_timeout(&mut events_rx, Duration::from_secs(5)).await;

    println!(
        "📊 Received {} events for single agent execution",
        events.len()
    );
    for (i, event) in events.iter().enumerate() {
        println!("  Event {}: {:?}", i + 1, event);
    }

    // Assertions
    assert!(
        !events.is_empty(),
        "Should receive events from pipeline execution"
    );

    assert!(
        assert_has_process_started(&events),
        "Should have ProcessStarted event"
    );

    assert!(
        assert_has_status_update(&events, ProcessStatus::Running),
        "Should transition to Running status"
    );

    assert!(
        assert_has_process_completed(&events),
        "Should complete successfully"
    );

    assert!(
        assert_has_log_chunks(&events),
        "Should have log chunks from agent execution"
    );

    // Verify event sequence
    assert_event_sequence(&events);

    // Verify process ID consistency
    let extracted_id = extract_process_id(&events).expect("Should have process ID");
    assert_eq!(extracted_id, process_id, "Process ID should be consistent");
}

/// Priority 1.1.2: Multi-agent sequential execution
///
/// Acceptance criteria:
/// 1. Agents execute in the correct order
/// 2. Each agent produces logs in sequence
/// 3. current_step_index advances correctly
/// 4. All agents complete before final completion
#[tokio::test]
async fn test_multi_agent_sequential_execution() {
    // Given: A pipeline with 3 agents
    let agents = vec![
        create_test_agent("agent-1"),
        create_test_agent("agent-2"),
        create_test_agent("agent-3"),
    ];
    let agent_manager = AgentManager::new(agents);

    let steps = vec![
        ProcessStep::Agent("agent-1".to_string()),
        ProcessStep::Agent("agent-2".to_string()),
        ProcessStep::Agent("agent-3".to_string()),
    ];
    let pipeline = create_test_pipeline("multi-agent-pipeline", steps);

    // When: Execute the pipeline
    let (events_tx, mut events_rx) = mpsc::channel(100);
    let engine = PipelineEngine::new(agent_manager);

    let process = create_test_process("multi-agent-pipeline", ProcessStatus::Pending);

    tokio::spawn(async move {
        let _ = engine.run(&pipeline, process, events_tx).await;
    });

    // Then: Collect and verify events
    let events = collect_events_until_timeout(&mut events_rx, Duration::from_secs(5)).await;

    println!(
        "📊 Received {} events for multi-agent execution",
        events.len()
    );

    // Should have multiple log chunks (one per agent minimum)
    let log_count = count_log_chunks(&events);
    assert!(
        log_count >= 3,
        "Should have at least 3 log chunks (one per agent), got {}",
        log_count
    );

    // Verify completion
    assert!(
        assert_has_process_completed(&events),
        "Should complete after all agents finish"
    );

    // Verify event sequence
    assert_event_sequence(&events);
}

/// Priority 1.2.1 + 1.2.2: HUMAN_REVIEW pause and resume
///
/// Acceptance criteria:
/// 1. Pipeline pauses at HUMAN_REVIEW with StatusUpdate(HumanReview)
/// 2. Execution blocks waiting for resume signal
/// 3. StateManager.resume_process_by_id() triggers continuation
/// 4. "Resumed from human review" log appears
/// 5. Pipeline completes after resume
#[tokio::test]
async fn test_human_review_pause_and_resume() {
    // Given: A pipeline with HUMAN_REVIEW between agents
    let agents = vec![create_test_agent("agent-1"), create_test_agent("agent-2")];
    let agent_manager = AgentManager::new(agents);

    let steps = vec![
        ProcessStep::Agent("agent-1".to_string()),
        ProcessStep::HumanReview(pk_protocol::pipeline_models::HumanReviewMarker),
        ProcessStep::Agent("agent-2".to_string()),
    ];
    let pipeline = create_test_pipeline("review-pipeline", steps);

    // When: Start pipeline via StateManager
    let (events_tx, mut events_rx) = mpsc::channel(100);
    let state_manager = StateManager::new(agent_manager, events_tx);

    let process_id = state_manager.start_pipeline(pipeline).await;

    // Wait for HUMAN_REVIEW status
    let mut human_review_reached = false;
    let timeout = Duration::from_secs(3);

    let mut collected_events = Vec::new();
    let start = tokio::time::Instant::now();

    while start.elapsed() < timeout {
        if let Ok(Some(event)) =
            tokio::time::timeout(Duration::from_millis(100), events_rx.recv()).await
        {
            let is_human_review = matches!(
                &event,
                Event::ProcessStatusUpdate {
                    status: ProcessStatus::HumanReview,
                    ..
                }
            );
            collected_events.push(event);

            if is_human_review {
                human_review_reached = true;
                break;
            }
        }
    }

    println!("📊 Events before resume: {} events", collected_events.len());
    for (i, event) in collected_events.iter().enumerate() {
        println!("  Event {}: {:?}", i + 1, event);
    }

    assert!(
        human_review_reached,
        "Pipeline should pause at HUMAN_REVIEW"
    );

    // Then: Resume the process
    let resume_result = state_manager.resume_process_by_id(process_id).await;
    assert!(resume_result.is_ok(), "Resume should succeed");

    // Wait for completion
    let mut completed = false;
    let start = tokio::time::Instant::now();

    while start.elapsed() < timeout {
        if let Ok(Some(event)) =
            tokio::time::timeout(Duration::from_millis(100), events_rx.recv()).await
        {
            let is_completed = matches!(&event, Event::ProcessCompleted { .. });
            collected_events.push(event);

            if is_completed {
                completed = true;
                break;
            }
        }
    }

    println!(
        "📊 Total events after resume: {} events",
        collected_events.len()
    );

    assert!(completed, "Pipeline should complete after resume");

    // Verify we got ProcessResumed event
    let has_resumed = collected_events
        .iter()
        .any(|e| matches!(e, Event::ProcessResumed { .. }));
    assert!(has_resumed, "Should emit ProcessResumed event");

    // Verify final process state
    let final_process = state_manager.get_process(process_id).await;
    assert!(final_process.is_some(), "Process should exist");
    assert_eq!(
        final_process.unwrap().status,
        ProcessStatus::Completed,
        "Process should be completed"
    );
}

/// Priority 2.1.1: Agent failure stops pipeline
///
/// Acceptance criteria:
/// 1. When an agent fails, StatusUpdate(Failed) is emitted
/// 2. Error message is logged in ProcessError event
/// 3. Pipeline execution stops immediately
/// 4. Subsequent agents are NOT executed
#[tokio::test]
async fn test_agent_failure_stops_pipeline() {
    // Given: A pipeline where the second agent fails
    let agents = vec![
        create_test_agent("agent-1"),
        create_failure_agent("failing-agent"),
        create_test_agent("agent-3"),
    ];
    let agent_manager = AgentManager::new(agents);

    let steps = vec![
        ProcessStep::Agent("agent-1".to_string()),
        ProcessStep::Agent("failing-agent".to_string()),
        ProcessStep::Agent("agent-3".to_string()),
    ];
    let pipeline = create_test_pipeline("failing-pipeline", steps);

    // When: Execute the pipeline
    let (events_tx, mut events_rx) = mpsc::channel(100);
    let engine = PipelineEngine::new(agent_manager);

    let process = create_test_process("failing-pipeline", ProcessStatus::Pending);

    tokio::spawn(async move {
        let _ = engine.run(&pipeline, process, events_tx).await;
    });

    // Then: Collect and verify events
    let events = collect_events_until_timeout(&mut events_rx, Duration::from_secs(5)).await;

    println!("📊 Received {} events for failing pipeline", events.len());
    for (i, event) in events.iter().enumerate() {
        println!("  Event {}: {:?}", i + 1, event);
    }

    // Should transition to Failed status
    assert!(
        assert_has_status_update(&events, ProcessStatus::Failed),
        "Should transition to Failed status"
    );

    // Should have ProcessError event
    let has_error = events
        .iter()
        .any(|e| matches!(e, Event::ProcessError { .. }));
    assert!(has_error, "Should emit ProcessError event");

    // Should NOT have ProcessCompleted
    assert!(
        !assert_has_process_completed(&events),
        "Should NOT complete successfully"
    );

    // Verify that agent-3 was NOT executed by checking logs
    let logs: Vec<String> = events
        .iter()
        .filter_map(|e| match e {
            Event::ProcessLogChunk { content, .. } => Some(content.clone()),
            _ => None,
        })
        .collect();

    let has_agent3_log = logs.iter().any(|log| log.contains("agent-3"));
    assert!(
        !has_agent3_log,
        "Agent-3 should NOT be executed after failure"
    );
}

/// Priority 1.3.1: Event sequence validation
///
/// Acceptance criteria:
/// 1. ProcessStarted is ALWAYS the first event
/// 2. StatusUpdate(Running) comes before log chunks
/// 3. ProcessCompleted is the last event (for successful execution)
/// 4. No duplicate events
/// 5. Process state fields are correctly populated
#[tokio::test]
async fn test_event_sequence_validation() {
    // Given: A simple pipeline
    let agents = vec![create_test_agent("agent-1")];
    let agent_manager = AgentManager::new(agents);

    let steps = vec![ProcessStep::Agent("agent-1".to_string())];
    let pipeline = create_test_pipeline("sequence-test", steps);

    // When: Execute the pipeline
    let (events_tx, mut events_rx) = mpsc::channel(100);
    let engine = PipelineEngine::new(agent_manager);

    let process = create_test_process("sequence-test", ProcessStatus::Pending);
    let process_id = process.id;

    tokio::spawn(async move {
        let _ = engine.run(&pipeline, process, events_tx).await;
    });

    // Then: Collect and verify events
    let events = collect_events_until_timeout(&mut events_rx, Duration::from_secs(5)).await;

    println!("📊 Event sequence validation: {} events", events.len());
    for (i, event) in events.iter().enumerate() {
        println!("  [{:02}] {:?}", i + 1, event);
    }

    // Rule 1: ProcessStarted is first
    assert!(
        matches!(events.first(), Some(Event::ProcessStarted { .. })),
        "First event must be ProcessStarted"
    );

    // Rule 2: Find indices of key events
    let started_idx = events
        .iter()
        .position(|e| matches!(e, Event::ProcessStarted { .. }))
        .expect("Should have ProcessStarted");

    let running_idx = events
        .iter()
        .position(|e| {
            matches!(
                e,
                Event::ProcessStatusUpdate {
                    status: ProcessStatus::Running,
                    ..
                }
            )
        })
        .expect("Should have Running status");

    let first_log_idx = events
        .iter()
        .position(|e| matches!(e, Event::ProcessLogChunk { .. }));

    let completed_idx = events
        .iter()
        .position(|e| matches!(e, Event::ProcessCompleted { .. }))
        .expect("Should have ProcessCompleted");

    // Rule 3: Verify ordering
    assert!(
        started_idx < running_idx,
        "ProcessStarted should come before Running status"
    );

    if let Some(log_idx) = first_log_idx {
        assert!(
            running_idx < log_idx,
            "Running status should come before log chunks"
        );
        assert!(
            log_idx < completed_idx,
            "Log chunks should come before ProcessCompleted"
        );
    }

    // Rule 4: Last event is ProcessCompleted
    assert!(
        matches!(events.last(), Some(Event::ProcessCompleted { .. })),
        "Last event must be ProcessCompleted"
    );

    // Rule 5: Process ID consistency
    for event in &events {
        match event {
            Event::ProcessStarted { process_id: id, .. }
            | Event::ProcessStatusUpdate { process_id: id, .. }
            | Event::ProcessLogChunk { process_id: id, .. }
            | Event::ProcessCompleted { process_id: id } => {
                assert_eq!(
                    *id, process_id,
                    "All events should have the same process_id"
                );
            }
            _ => {}
        }
    }

    println!("✅ Event sequence validation passed");
}

/// Additional test: Empty pipeline (edge case)
///
/// Tests that a pipeline with no steps completes immediately.
#[tokio::test]
async fn test_empty_pipeline_completes_immediately() {
    // Given: An empty pipeline
    let agents = vec![];
    let agent_manager = AgentManager::new(agents);

    let steps: Vec<ProcessStep> = vec![];
    let pipeline = create_test_pipeline("empty-pipeline", steps);

    // When: Execute the pipeline
    let (events_tx, mut events_rx) = mpsc::channel(100);
    let engine = PipelineEngine::new(agent_manager);

    let process = create_test_process("empty-pipeline", ProcessStatus::Pending);

    tokio::spawn(async move {
        let _ = engine.run(&pipeline, process, events_tx).await;
    });

    // Then: Should complete immediately
    let events = collect_events_until_timeout(&mut events_rx, Duration::from_secs(2)).await;

    println!("📊 Empty pipeline events: {} events", events.len());

    assert!(
        assert_has_process_started(&events),
        "Should start even with no steps"
    );

    assert!(
        assert_has_process_completed(&events),
        "Should complete immediately"
    );

    // Should have minimal events: Started, StatusUpdate(Running), StatusUpdate(Completed), Completed
    assert!(
        events.len() >= 3,
        "Should have at least 3 events (Started, Running, Completed)"
    );
}

/// Additional test: Multiple HUMAN_REVIEW steps
///
/// Tests sequential pause-resume cycles.
#[tokio::test]
async fn test_multiple_human_review_steps() {
    // Given: A pipeline with multiple HUMAN_REVIEW steps
    let agents = vec![
        create_test_agent("agent-1"),
        create_test_agent("agent-2"),
        create_test_agent("agent-3"),
    ];
    let agent_manager = AgentManager::new(agents);

    let steps = vec![
        ProcessStep::Agent("agent-1".to_string()),
        ProcessStep::HumanReview(pk_protocol::pipeline_models::HumanReviewMarker),
        ProcessStep::Agent("agent-2".to_string()),
        ProcessStep::HumanReview(pk_protocol::pipeline_models::HumanReviewMarker),
        ProcessStep::Agent("agent-3".to_string()),
    ];
    let pipeline = create_test_pipeline("multi-review-pipeline", steps);

    // When: Start pipeline
    let (events_tx, mut events_rx) = mpsc::channel(100);
    let state_manager = StateManager::new(agent_manager, events_tx);

    let process_id = state_manager.start_pipeline(pipeline).await;

    // Wait for first HUMAN_REVIEW
    let mut first_review_reached = false;
    let timeout = Duration::from_secs(2);
    let start = tokio::time::Instant::now();

    while start.elapsed() < timeout {
        if let Ok(Some(event)) =
            tokio::time::timeout(Duration::from_millis(100), events_rx.recv()).await
        {
            if matches!(
                &event,
                Event::ProcessStatusUpdate {
                    status: ProcessStatus::HumanReview,
                    ..
                }
            ) {
                first_review_reached = true;
                break;
            }
        }
    }

    assert!(first_review_reached, "Should reach first HUMAN_REVIEW");

    // Resume first time
    state_manager
        .resume_process_by_id(process_id)
        .await
        .unwrap();

    // Wait for second HUMAN_REVIEW
    let mut second_review_reached = false;
    let start = tokio::time::Instant::now();

    while start.elapsed() < timeout {
        if let Ok(Some(event)) =
            tokio::time::timeout(Duration::from_millis(100), events_rx.recv()).await
        {
            if matches!(
                &event,
                Event::ProcessStatusUpdate {
                    status: ProcessStatus::HumanReview,
                    ..
                }
            ) {
                second_review_reached = true;
                break;
            }
        }
    }

    assert!(second_review_reached, "Should reach second HUMAN_REVIEW");

    // Resume second time
    state_manager
        .resume_process_by_id(process_id)
        .await
        .unwrap();

    // Wait for completion
    let mut completed = false;
    let start = tokio::time::Instant::now();

    while start.elapsed() < timeout {
        if let Ok(Some(event)) =
            tokio::time::timeout(Duration::from_millis(100), events_rx.recv()).await
        {
            if matches!(&event, Event::ProcessCompleted { .. }) {
                completed = true;
                break;
            }
        }
    }

    assert!(completed, "Should complete after second resume");

    println!("✅ Multiple HUMAN_REVIEW test passed");
}
