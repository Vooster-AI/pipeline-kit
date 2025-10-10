//! # pk-tui
//!
//! Terminal User Interface for pipeline-kit.
//!
//! This crate provides the interactive TUI for monitoring and controlling
//! pipeline execution. It communicates with `pk-core` via channels using
//! the `Op` and `Event` protocol defined in `pk-protocol`.

pub mod app;
pub mod event_handler;
pub mod tui;
pub mod widgets;

pub use app::App;
pub use tui::Tui;

use anyhow::Result;
use tokio::sync::mpsc;

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

    // Create channels for communication between UI and core
    // Note: In a full implementation, these would connect to pk-core
    let (op_tx, mut _op_rx) = mpsc::unbounded_channel();
    let (_event_tx, event_rx) = mpsc::unbounded_channel();

    // Create and run the app
    let mut app = App::new(op_tx, event_rx);
    let result = app.run(&mut tui).await;

    // Restore terminal before returning
    tui.restore()?;

    result
}
