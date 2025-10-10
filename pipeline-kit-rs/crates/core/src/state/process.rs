//! Process state machine implementation.
//!
//! This module provides functions for managing the lifecycle of a Process,
//! including state transitions and event emission.

use pk_protocol::ipc::Event;
use pk_protocol::process_models::{Process, ProcessStatus};
use tokio::sync::mpsc::Sender;
use uuid::Uuid;

/// Create a new Process with Pending status.
///
/// # Arguments
///
/// * `pipeline_name` - The name of the pipeline to execute
///
/// # Returns
///
/// A new Process instance with a unique ID and Pending status.
pub fn create_process(pipeline_name: String) -> Process {
    Process {
        id: Uuid::new_v4(),
        pipeline_name,
        status: ProcessStatus::Pending,
        current_step: 0,
        logs: Vec::new(),
    }
}

/// Transition the process to Running status and emit event.
///
/// # Arguments
///
/// * `process` - The process to start
/// * `events_tx` - Channel to send status update events
pub async fn start_process(process: &mut Process, events_tx: &Sender<Event>) {
    process.status = ProcessStatus::Running;
    let _ = events_tx
        .send(Event::ProcessStatusUpdate {
            process_id: process.id,
            status: process.status,
            step_index: process.current_step,
        })
        .await;
}

/// Transition to HumanReview status and emit event.
///
/// This is called when a HUMAN_REVIEW step is encountered.
///
/// # Arguments
///
/// * `process` - The process to pause
/// * `events_tx` - Channel to send status update events
pub async fn pause_for_human_review(process: &mut Process, events_tx: &Sender<Event>) {
    process.status = ProcessStatus::HumanReview;
    let _ = events_tx
        .send(Event::ProcessStatusUpdate {
            process_id: process.id,
            status: process.status,
            step_index: process.current_step,
        })
        .await;
}

/// Transition to Paused status and emit event.
///
/// This is called when the user manually pauses the process.
///
/// # Arguments
///
/// * `process` - The process to pause
/// * `events_tx` - Channel to send status update events
pub async fn pause_process(process: &mut Process, events_tx: &Sender<Event>) {
    process.status = ProcessStatus::Paused;
    let _ = events_tx
        .send(Event::ProcessStatusUpdate {
            process_id: process.id,
            status: process.status,
            step_index: process.current_step,
        })
        .await;
}

/// Resume from Paused or HumanReview status to Running.
///
/// # Arguments
///
/// * `process` - The process to resume
/// * `events_tx` - Channel to send status update events
pub async fn resume_process(process: &mut Process, events_tx: &Sender<Event>) {
    process.status = ProcessStatus::Running;
    let _ = events_tx
        .send(Event::ProcessStatusUpdate {
            process_id: process.id,
            status: process.status,
            step_index: process.current_step,
        })
        .await;
}

/// Mark the process as completed and emit event.
///
/// # Arguments
///
/// * `process` - The process to complete
/// * `events_tx` - Channel to send completion event
pub async fn complete_process(process: &mut Process, events_tx: &Sender<Event>) {
    process.status = ProcessStatus::Completed;
    let _ = events_tx
        .send(Event::ProcessStatusUpdate {
            process_id: process.id,
            status: process.status,
            step_index: process.current_step,
        })
        .await;
    let _ = events_tx
        .send(Event::ProcessCompleted {
            process_id: process.id,
        })
        .await;
}

/// Mark the process as failed and emit error event.
///
/// # Arguments
///
/// * `process` - The process to fail
/// * `events_tx` - Channel to send error event
/// * `error` - Error message describing the failure
pub async fn fail_process(process: &mut Process, events_tx: &Sender<Event>, error: String) {
    process.status = ProcessStatus::Failed;
    let _ = events_tx
        .send(Event::ProcessStatusUpdate {
            process_id: process.id,
            status: process.status,
            step_index: process.current_step,
        })
        .await;
    let _ = events_tx
        .send(Event::ProcessError {
            process_id: process.id,
            error,
        })
        .await;
}

/// Append a log message to the process logs and emit event.
///
/// # Arguments
///
/// * `process` - The process to log to
/// * `events_tx` - Channel to send log chunk event
/// * `message` - Log message to append
pub async fn log_to_process(process: &mut Process, events_tx: &Sender<Event>, message: String) {
    process.logs.push(message.clone());
    let _ = events_tx
        .send(Event::ProcessLogChunk {
            process_id: process.id,
            content: message,
        })
        .await;
}

/// Move to the next step in the pipeline.
///
/// # Arguments
///
/// * `process` - The process to advance
pub fn advance_step(process: &mut Process) {
    process.current_step += 1;
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_create_process() {
        let process = create_process("test-pipeline".to_string());
        assert_eq!(process.pipeline_name, "test-pipeline");
        assert_eq!(process.status, ProcessStatus::Pending);
        assert_eq!(process.current_step, 0);
        assert!(process.logs.is_empty());
    }

    #[tokio::test]
    async fn test_start_process() {
        let mut process = create_process("test-pipeline".to_string());
        let (tx, mut rx) = mpsc::channel(10);

        start_process(&mut process, &tx).await;

        assert_eq!(process.status, ProcessStatus::Running);

        let event = rx.recv().await.unwrap();
        assert!(matches!(
            event,
            Event::ProcessStatusUpdate {
                status: ProcessStatus::Running,
                step_index: 0,
                ..
            }
        ));
    }

    #[tokio::test]
    async fn test_pause_for_human_review() {
        let mut process = create_process("test-pipeline".to_string());
        let (tx, mut rx) = mpsc::channel(10);

        pause_for_human_review(&mut process, &tx).await;

        assert_eq!(process.status, ProcessStatus::HumanReview);

        let event = rx.recv().await.unwrap();
        assert!(matches!(
            event,
            Event::ProcessStatusUpdate {
                status: ProcessStatus::HumanReview,
                ..
            }
        ));
    }

    #[tokio::test]
    async fn test_complete_process() {
        let mut process = create_process("test-pipeline".to_string());
        let (tx, mut rx) = mpsc::channel(10);

        complete_process(&mut process, &tx).await;

        assert_eq!(process.status, ProcessStatus::Completed);

        // Should receive two events: StatusUpdate and Completed
        let event1 = rx.recv().await.unwrap();
        assert!(matches!(
            event1,
            Event::ProcessStatusUpdate {
                status: ProcessStatus::Completed,
                ..
            }
        ));

        let event2 = rx.recv().await.unwrap();
        assert!(matches!(event2, Event::ProcessCompleted { .. }));
    }

    #[tokio::test]
    async fn test_fail_process() {
        let mut process = create_process("test-pipeline".to_string());
        let (tx, mut rx) = mpsc::channel(10);

        fail_process(&mut process, &tx, "Test error".to_string()).await;

        assert_eq!(process.status, ProcessStatus::Failed);

        // Should receive two events: StatusUpdate and Error
        let event1 = rx.recv().await.unwrap();
        assert!(matches!(
            event1,
            Event::ProcessStatusUpdate {
                status: ProcessStatus::Failed,
                ..
            }
        ));

        let event2 = rx.recv().await.unwrap();
        assert!(matches!(event2, Event::ProcessError { error, .. } if error == "Test error"));
    }

    #[tokio::test]
    async fn test_log_to_process() {
        let mut process = create_process("test-pipeline".to_string());
        let (tx, mut rx) = mpsc::channel(10);

        log_to_process(&mut process, &tx, "Test log message".to_string()).await;

        assert_eq!(process.logs.len(), 1);
        assert_eq!(process.logs[0], "Test log message");

        let event = rx.recv().await.unwrap();
        assert!(matches!(
            event,
            Event::ProcessLogChunk { content, .. } if content == "Test log message"
        ));
    }

    #[tokio::test]
    async fn test_advance_step() {
        let mut process = create_process("test-pipeline".to_string());
        assert_eq!(process.current_step, 0);

        advance_step(&mut process);
        assert_eq!(process.current_step, 1);

        advance_step(&mut process);
        assert_eq!(process.current_step, 2);
    }

    #[tokio::test]
    async fn test_resume_process() {
        let mut process = create_process("test-pipeline".to_string());
        let (tx, mut rx) = mpsc::channel(10);

        // First pause
        pause_process(&mut process, &tx).await;
        assert_eq!(process.status, ProcessStatus::Paused);
        let _ = rx.recv().await;

        // Then resume
        resume_process(&mut process, &tx).await;
        assert_eq!(process.status, ProcessStatus::Running);

        let event = rx.recv().await.unwrap();
        assert!(matches!(
            event,
            Event::ProcessStatusUpdate {
                status: ProcessStatus::Running,
                ..
            }
        ));
    }
}
