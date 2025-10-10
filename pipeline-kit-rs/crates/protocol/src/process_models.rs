//! Runtime process state models.
//!
//! This module defines the structures for tracking the state of running
//! pipeline executions.

use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

/// Represents the current lifecycle status of a running pipeline process.
///
/// The status progresses through these states during normal execution:
/// Pending -> Running -> Completed
///
/// Special states:
/// - HumanReview: Paused waiting for manual review
/// - Paused: Manually paused by user
/// - Failed: Execution encountered an error
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, TS)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProcessStatus {
    /// Process has been created but not started yet.
    Pending,

    /// Process is actively executing.
    Running,

    /// Process has been manually paused by the user.
    Paused,

    /// Process is waiting for human review before continuing.
    ///
    /// This happens when a HUMAN_REVIEW step is encountered.
    HumanReview,

    /// Process has completed successfully.
    Completed,

    /// Process has failed due to an error.
    Failed,
}

/// Represents the runtime state of a single pipeline execution.
///
/// Each time a pipeline is started, a new Process instance is created
/// with a unique ID to track its execution state.
#[derive(Serialize, Deserialize, Debug, Clone, TS)]
pub struct Process {
    /// Unique identifier for this process execution.
    ///
    /// Generated when the process is created and used to track
    /// the process throughout its lifecycle.
    #[ts(type = "string")]
    pub id: Uuid,

    /// Name of the pipeline being executed.
    ///
    /// References a pipeline defined in `.pipeline-kit/pipelines/*.yaml`.
    pub pipeline_name: String,

    /// Current execution status.
    pub status: ProcessStatus,

    /// Zero-based index of the current step in the pipeline.
    ///
    /// Points to the step currently being executed or the next step
    /// to be executed if paused.
    pub current_step: usize,

    /// Accumulated log messages from this process execution.
    ///
    /// Contains output from agents, status updates, and error messages.
    pub logs: Vec<String>,
}
