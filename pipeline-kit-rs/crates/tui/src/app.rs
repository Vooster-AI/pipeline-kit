//! TUI application state and event loop.
//!
//! This module defines the main `App` struct that manages the TUI state
//! and the event loop using `tokio::select!`.

use anyhow::Result;
use crossterm::event::KeyEvent;
use pk_protocol::{Event, Op, Process};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use tokio::select;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio_stream::StreamExt;

use crate::event_handler;
use crate::tui::{Tui, TuiEvent};

/// Main TUI application state.
///
/// This struct holds all the state needed to render the UI and process events.
pub struct App {
    /// List of all processes being tracked.
    pub processes: Vec<Process>,
    /// Index of the currently selected process (for detail view).
    pub selected_index: usize,
    /// Current command input from the user.
    pub command_input: String,
    /// Channel to send operations to the core.
    pub op_tx: UnboundedSender<Op>,
    /// Channel to receive events from the core.
    pub event_rx: UnboundedReceiver<Event>,
    /// Flag to indicate if the application should exit.
    pub should_exit: bool,
}

impl App {
    /// Create a new App with communication channels.
    pub fn new(
        op_tx: UnboundedSender<Op>,
        event_rx: UnboundedReceiver<Event>,
    ) -> Self {
        Self {
            processes: Vec::new(),
            selected_index: 0,
            command_input: String::new(),
            op_tx,
            event_rx,
            should_exit: false,
        }
    }

    /// Main event loop.
    ///
    /// Uses `tokio::select!` to handle keyboard input and core events concurrently.
    pub async fn run(&mut self, tui: &mut Tui) -> Result<()> {
        let mut tui_events = tui.event_stream();

        tui.frame_requester().schedule_frame();

        while !self.should_exit {
            select! {
                Some(event) = self.event_rx.recv() => {
                    self.handle_core_event(event);
                    tui.frame_requester().schedule_frame();
                }
                Some(tui_event) = tui_events.next() => {
                    self.handle_tui_event(tui, tui_event).await?;
                }
            }
        }

        Ok(())
    }

    /// Handle events from the core (pk-core).
    fn handle_core_event(&mut self, event: Event) {
        event_handler::handle_core_event(&mut self.processes, event);
    }

    /// Handle TUI events (keyboard input, resize, draw).
    async fn handle_tui_event(&mut self, tui: &mut Tui, event: TuiEvent) -> Result<()> {
        match event {
            TuiEvent::Key(key_event) => {
                self.handle_key_event(key_event);
                tui.frame_requester().schedule_frame();
            }
            TuiEvent::Paste(_) => {
                // Ignore paste events for now
            }
            TuiEvent::Draw => {
                tui.draw(|frame| {
                    self.render(frame);
                })?;
            }
        }
        Ok(())
    }

    /// Handle keyboard events.
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        self.should_exit = event_handler::handle_keyboard_event(
            key_event,
            &mut self.command_input,
            &mut self.selected_index,
            &self.processes,
            &self.op_tx,
        );
    }

    /// Render the TUI.
    fn render(&self, frame: &mut Frame) {
        let area = frame.area();

        // Create a 3-panel layout: dashboard (top), detail (middle), command (bottom)
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(40),  // Dashboard
                Constraint::Percentage(50),  // Detail
                Constraint::Length(3),        // Command input
            ])
            .split(area);

        self.render_dashboard(frame, chunks[0]);
        self.render_detail(frame, chunks[1]);
        self.render_command_input(frame, chunks[2]);
    }

    /// Render the dashboard (list of processes).
    fn render_dashboard(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Dashboard - Processes");

        let text = if self.processes.is_empty() {
            "No processes running.".to_string()
        } else {
            self.processes
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    let prefix = if i == self.selected_index { "> " } else { "  " };
                    format!("{}{} | {:?} | {}", prefix, p.pipeline_name, p.status, p.id)
                })
                .collect::<Vec<_>>()
                .join("\n")
        };

        let paragraph = Paragraph::new(text).block(block);
        frame.render_widget(paragraph, area);
    }

    /// Render the detail view (selected process logs).
    fn render_detail(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Detail - Process Logs");

        let text = if let Some(process) = self.processes.get(self.selected_index) {
            if process.logs.is_empty() {
                "No logs yet.".to_string()
            } else {
                process.logs.join("\n")
            }
        } else {
            "No process selected.".to_string()
        };

        let paragraph = Paragraph::new(text).block(block);
        frame.render_widget(paragraph, area);
    }

    /// Render the command input area.
    fn render_command_input(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Command (q to quit)");

        let text = format!("> {}", self.command_input);
        let paragraph = Paragraph::new(text)
            .block(block)
            .style(Style::default().fg(Color::Yellow));
        frame.render_widget(paragraph, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyCode;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use tokio::sync::mpsc::unbounded_channel;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_app_renders_empty_screen() {
        let (op_tx, _op_rx) = unbounded_channel();
        let (_event_tx, event_rx) = unbounded_channel();

        let app = App::new(op_tx, event_rx);

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                app.render(frame);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();

        // Check that the dashboard title is rendered
        let dashboard_title = buffer.content().iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        assert!(dashboard_title.contains("Dashboard"));
        assert!(dashboard_title.contains("Detail"));
        assert!(dashboard_title.contains("Command"));
    }

    #[tokio::test]
    async fn test_app_quit_on_q() {
        let (op_tx, _op_rx) = unbounded_channel();
        let (_event_tx, event_rx) = unbounded_channel();

        let mut app = App::new(op_tx, event_rx);

        assert!(!app.should_exit);

        app.handle_key_event(KeyEvent::from(KeyCode::Char('q')));

        assert!(app.should_exit);
    }

    #[tokio::test]
    async fn test_app_handles_process_started_event() {
        let (op_tx, _op_rx) = unbounded_channel();
        let (_event_tx, event_rx) = unbounded_channel();

        let mut app = App::new(op_tx, event_rx);

        assert_eq!(app.processes.len(), 0);

        let process_id = Uuid::new_v4();
        app.handle_core_event(Event::ProcessStarted {
            process_id,
            pipeline_name: "test-pipeline".to_string(),
        });

        assert_eq!(app.processes.len(), 1);
        assert_eq!(app.processes[0].pipeline_name, "test-pipeline");
        assert_eq!(app.processes[0].id, process_id);
    }

    #[tokio::test]
    async fn test_app_navigation_with_arrow_keys() {
        let (op_tx, _op_rx) = unbounded_channel();
        let (_event_tx, event_rx) = unbounded_channel();

        let mut app = App::new(op_tx, event_rx);

        // Add some test processes
        for i in 0..3 {
            app.handle_core_event(Event::ProcessStarted {
                process_id: Uuid::new_v4(),
                pipeline_name: format!("pipeline-{}", i),
            });
        }

        assert_eq!(app.selected_index, 0);

        app.handle_key_event(KeyEvent::from(KeyCode::Down));
        assert_eq!(app.selected_index, 1);

        app.handle_key_event(KeyEvent::from(KeyCode::Down));
        assert_eq!(app.selected_index, 2);

        // Should not go beyond the last index
        app.handle_key_event(KeyEvent::from(KeyCode::Down));
        assert_eq!(app.selected_index, 2);

        app.handle_key_event(KeyEvent::from(KeyCode::Up));
        assert_eq!(app.selected_index, 1);

        app.handle_key_event(KeyEvent::from(KeyCode::Up));
        assert_eq!(app.selected_index, 0);

        // Should not go below 0
        app.handle_key_event(KeyEvent::from(KeyCode::Up));
        assert_eq!(app.selected_index, 0);
    }
}
