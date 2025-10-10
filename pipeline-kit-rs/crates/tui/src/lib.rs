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

pub use app::App;
pub use tui::Tui;
