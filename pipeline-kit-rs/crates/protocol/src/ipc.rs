//! Inter-process communication protocol.
//!
//! This module defines the message types for asynchronous communication
//! between the TUI (user interface) and the Core (business logic).
//!
//! The protocol follows an Operation/Event pattern:
//! - `Op`: Commands sent from TUI to Core
//! - `Event`: Status updates sent from Core to TUI
//!
//! Communication is asynchronous and channel-based, allowing the UI to
//! remain responsive while the core processes pipeline executions.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use ts_rs::TS;
use uuid::Uuid;

use crate::process_models::ProcessStatus;

/// Operations sent from the UI (TUI) to the Core logic.
///
/// These represent user commands and requests for information.
/// The core processes these operations and responds with Events.
///
/// Uses tagged enum serialization for TypeScript compatibility:
/// ```json
/// {
///   "type": "startPipeline",
///   "payload": {
///     "name": "my-pipeline",
///     "reference_file": "/path/to/ref.md"
///   }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "type", content = "payload", rename_all = "camelCase")]
pub enum Op {
    /// Start executing a pipeline.
    ///
    /// Initiates a new pipeline execution with an optional reference file.
    StartPipeline {
        /// Name of the pipeline to execute.
        name: String,
        /// Optional path to a reference file for context.
        reference_file: Option<PathBuf>,
    },

    /// Pause a running process.
    ///
    /// The process will stop after completing its current step.
    PauseProcess {
        #[ts(type = "string")]
        process_id: Uuid,
    },

    /// Resume a paused process.
    ///
    /// The process will continue from where it was paused.
    ResumeProcess {
        #[ts(type = "string")]
        process_id: Uuid,
    },

    /// Terminate a process immediately.
    ///
    /// The process will be stopped and marked as failed.
    KillProcess {
        #[ts(type = "string")]
        process_id: Uuid,
    },

    /// Request the current state of all processes.
    ///
    /// Core will respond with dashboard state information.
    GetDashboardState,

    /// Request detailed information about a specific process.
    GetProcessDetail {
        #[ts(type = "string")]
        process_id: Uuid,
    },

    /// Shut down the application gracefully.
    ///
    /// All running processes will be terminated.
    Shutdown,
}

/// Events sent from the Core logic to the UI (TUI).
///
/// These represent state changes and status updates that the UI should
/// reflect to the user.
///
/// Uses tagged enum serialization for TypeScript compatibility:
/// ```json
/// {
///   "type": "processStatusUpdate",
///   "payload": {
///     "process_id": "uuid-here",
///     "status": "RUNNING",
///     "step_index": 2
///   }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "type", content = "payload", rename_all = "camelCase")]
pub enum Event {
    /// A new process has been started.
    ProcessStarted {
        #[ts(type = "string")]
        process_id: Uuid,
        pipeline_name: String,
    },

    /// A process's status has changed.
    ProcessStatusUpdate {
        #[ts(type = "string")]
        process_id: Uuid,
        status: ProcessStatus,
        step_index: usize,
    },

    /// A process has produced new log output.
    ///
    /// The TUI should append this to the process's log display.
    ProcessLogChunk {
        #[ts(type = "string")]
        process_id: Uuid,
        content: String,
    },

    /// A process has completed successfully.
    ProcessCompleted {
        #[ts(type = "string")]
        process_id: Uuid,
    },

    /// A process has encountered an error.
    ProcessError {
        #[ts(type = "string")]
        process_id: Uuid,
        error: String,
    },

    /// A process was killed by user request.
    ProcessKilled {
        #[ts(type = "string")]
        process_id: Uuid,
    },

    /// A process was resumed from paused state.
    ProcessResumed {
        #[ts(type = "string")]
        process_id: Uuid,
    },
}
