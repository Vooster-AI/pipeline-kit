//! # pk-tui
//!
//! Terminal User Interface for pipeline-kit.
//!
//! This crate provides the interactive TUI for monitoring and controlling
//! pipeline execution. It communicates with `pk-core` via channels using
//! the `Op` and `Event` protocol defined in `pk-protocol`.

pub mod app;
pub mod event;
pub mod event_handler;
pub mod tui;
pub mod widgets;

pub use app::App;
pub use event::EventStatus;
pub use tui::Tui;

use anyhow::Result;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

// Core wiring: load config, manage agents/state, and speak protocol
use pk_core::agents::manager::AgentManager;
use pk_core::config::loader::load_config;
use pk_core::state::manager::StateManager;
use pk_protocol::ipc::{Event, Op};

/// Run the TUI application.
///
/// This is the main entry point for the interactive TUI. It:
/// 1. Sets up the terminal in raw mode
/// 2. Creates communication channels between the UI and core
/// 3. Runs the main event loop
/// 4. Restores the terminal on exit
///
/// # Errors
///
/// Returns an error if terminal initialization fails or if the event loop
/// encounters an unrecoverable error.
pub async fn run_app() -> Result<()> {
    // Initialize the terminal
    let mut tui = Tui::init()?;

    // Clear the terminal
    tui.clear()?;

    // Load configuration from current working directory
    let root = std::env::current_dir()?;
    let config = load_config(&root).await?;

    // Initialize agent manager from config
    let agent_manager = AgentManager::new(config.agents.clone());

    // Bridge channels between Core (bounded) and TUI (unbounded)
    // - Core emits Events on a bounded channel expected by StateManager
    // - TUI consumes Events on an unbounded channel used by App
    let (core_event_tx, mut core_event_rx) = mpsc::channel::<Event>(100);
    let (ui_event_tx, ui_event_rx) = mpsc::unbounded_channel::<Event>();

    // State manager drives pipeline/process lifecycle and emits events
    let state_manager = StateManager::new(agent_manager, core_event_tx);

    // UI sends Ops on an unbounded channel that Core will consume
    let (ui_op_tx, mut ui_op_rx) = mpsc::unbounded_channel::<Op>();

    // Forward Core events to the UI channel
    let _events_forwarder: JoinHandle<()> = tokio::spawn(async move {
        while let Some(ev) = core_event_rx.recv().await {
            // Best-effort forward; UI may have been dropped
            let _ = ui_event_tx.send(ev);
        }
    });

    // Handle Ops from the UI by invoking StateManager
    let state_manager_for_ops = state_manager;
    let pipelines = config.pipelines.clone();
    let _ops_handler: JoinHandle<()> = tokio::spawn(async move {
        while let Some(op) = ui_op_rx.recv().await {
            match op {
                Op::StartPipeline { name, .. } => {
                    if let Some(pipeline_def) = pipelines.iter().find(|p| p.name == name).cloned() {
                        // Fire-and-forget; StateManager emits lifecycle events
                        let _id = state_manager_for_ops.start_pipeline(pipeline_def).await;
                    } else {
                        // No pipeline found: emit a ProcessError-like notice
                        // Without a process_id, we cannot use ProcessError; skip emission.
                    }
                }
                Op::PauseProcess { process_id } => {
                    let _ = state_manager_for_ops.pause_process_by_id(process_id).await;
                }
                Op::ResumeProcess { process_id } => {
                    let _ = state_manager_for_ops.resume_process_by_id(process_id).await;
                }
                Op::KillProcess { process_id } => {
                    let _ = state_manager_for_ops.kill_process(process_id).await;
                }
                Op::GetDashboardState => {
                    // Optional: could emit synthetic events or a summary; no-op for now.
                }
                Op::GetProcessDetail { .. } => {
                    // Optional: detail query not implemented yet.
                }
                Op::Shutdown => {
                    // Shutdown handled by TUI (quit keys); no-op here.
                }
            }
        }
    });

    // Create and run the app
    let mut app = App::new(ui_op_tx, ui_event_rx);
    let result = app.run(&mut tui).await;

    // Restore terminal before returning
    tui.restore()?;

    result
}
