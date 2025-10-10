//! Main entry point for the pk-tui binary.
//!
//! This executable provides a standalone TUI for pipeline-kit.

use pk_tui::run_app;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize and run the TUI application
    run_app().await
}
