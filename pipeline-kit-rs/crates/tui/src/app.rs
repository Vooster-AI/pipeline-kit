//! TUI application state and event loop.
//!
//! This module defines the main `App` struct that manages the TUI state
//! and the event loop using `tokio::select!`.

use anyhow::Result;
use crossterm::event::KeyEvent;
use pk_protocol::Event;
use pk_protocol::Op;
use pk_protocol::Process;
use ratatui::layout::Constraint;
use ratatui::layout::Direction;
use ratatui::layout::Layout;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Paragraph;
use ratatui::Frame;
use tokio::select;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::UnboundedSender;
use tokio_stream::StreamExt;

use crate::event::EventStatus;
use crate::event_handler;
use crate::tui::Tui;
use crate::tui::TuiEvent;
use crate::widgets::dashboard;
use crate::widgets::CommandComposer;

/// Main TUI application state.
///
/// This struct holds all the state needed to render the UI and process events.
pub struct App {
    /// List of all processes being tracked.
    pub processes: Vec<Process>,
    /// Index of the currently selected process (for detail view).
    pub selected_index: usize,
    /// Command composer widget with autocomplete.
    pub command_composer: CommandComposer,
    /// Channel to send operations to the core.
    pub op_tx: UnboundedSender<Op>,
    /// Channel to receive events from the core.
    pub event_rx: UnboundedReceiver<Event>,
    /// Flag to indicate if the application should exit.
    pub should_exit: bool,
    /// Error message to display (if any).
    pub error_message: Option<String>,
}

impl App {
    /// Create a new App with communication channels.
    pub fn new(op_tx: UnboundedSender<Op>, event_rx: UnboundedReceiver<Event>) -> Self {
        Self {
            processes: Vec::new(),
            selected_index: 0,
            command_composer: CommandComposer::new(),
            op_tx,
            event_rx,
            should_exit: false,
            error_message: None,
        }
    }

    /// Create a new App with communication channels and pipeline names.
    pub fn with_pipelines(
        op_tx: UnboundedSender<Op>,
        event_rx: UnboundedReceiver<Event>,
        pipeline_names: Vec<String>,
    ) -> Self {
        Self {
            processes: Vec::new(),
            selected_index: 0,
            command_composer: CommandComposer::with_pipelines(pipeline_names),
            op_tx,
            event_rx,
            should_exit: false,
            error_message: None,
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
    ///
    /// This method implements the chain of responsibility pattern:
    /// 1. First, try to delegate the event to the CommandComposer widget
    /// 2. If the widget consumed the event, we're done
    /// 3. If not, handle global events (quit, process navigation, command submission)
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        use crossterm::event::KeyCode;
        use crossterm::event::KeyEventKind;
        use crossterm::event::KeyModifiers;

        // Ignore key release events
        if key_event.kind != KeyEventKind::Press {
            return;
        }

        // First, try to delegate to CommandComposer
        let status = self.command_composer.handle_key_event(key_event);

        // If the event was consumed by the widget, clear error on char input and return
        if status == EventStatus::Consumed {
            // Clear error message when user types (only for char input)
            if matches!(key_event.code, KeyCode::Char(_)) {
                self.error_message = None;
            }
            // Also clear error on Esc
            if matches!(key_event.code, KeyCode::Esc) {
                self.error_message = None;
            }
            return;
        }

        // Event was not consumed by widget, handle global events
        match key_event.code {
            // Global quit keys
            KeyCode::Char('q') if self.command_composer.input().is_empty() => {
                self.should_exit = true;
            }
            KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_exit = true;
            }

            // Process navigation (only when popup is not shown)
            KeyCode::Up => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
            }
            KeyCode::Down => {
                if self.selected_index + 1 < self.processes.len() {
                    self.selected_index += 1;
                }
            }

            // Command submission
            KeyCode::Enter => {
                self.handle_command_submit();
            }

            _ => {}
        }
    }

    /// Handle command submission (Enter key).
    fn handle_command_submit(&mut self) {
        match self.command_composer.parse_command() {
            Ok(Some(op)) => {
                // Send the Op to the core
                if let Err(e) = self.op_tx.send(op) {
                    self.error_message = Some(format!("Failed to send command: {}", e));
                } else {
                    // Clear the composer on success
                    self.command_composer.clear();
                    self.error_message = None;
                }
            }
            Ok(None) => {
                // Empty command, just clear
                self.command_composer.clear();
            }
            Err(err) => {
                // Show error message
                self.error_message = Some(err);
            }
        }
    }

    /// Render the TUI.
    fn render(&self, frame: &mut Frame) {
        let area = frame.area();

        // Create a 3-panel layout: dashboard (top), detail (middle), command (bottom)
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(40), // Dashboard
                Constraint::Percentage(50), // Detail
                Constraint::Length(3),      // Command input
            ])
            .split(area);

        self.render_dashboard(frame, chunks[0]);
        self.render_detail(frame, chunks[1]);
        self.render_command_input(frame, chunks[2]);
    }

    /// Render the dashboard (list of processes).
    fn render_dashboard(&self, frame: &mut Frame, area: Rect) {
        // Delegate to the dashboard widget's render_dashboard function
        dashboard::render_dashboard(frame, area, &self.processes, self.selected_index);
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
        // Render the composer input
        self.command_composer.render(area, frame.buffer_mut());

        // Render autocomplete popup if needed (above the command input)
        if self.command_composer.should_show_popup() {
            let popup_height = 7.min(area.height.saturating_sub(1));
            let popup_y = area.y.saturating_sub(popup_height);
            let popup_area = Rect {
                x: area.x,
                y: popup_y,
                width: area.width,
                height: popup_height,
            };
            self.command_composer
                .render_popup(popup_area, frame.buffer_mut());
        }

        // Render error message if any
        if let Some(ref error) = self.error_message {
            let error_text = format!("Error: {}", error);
            let error_paragraph = Paragraph::new(error_text).style(Style::default().fg(Color::Red));
            let error_area = Rect {
                x: area.x + 2,
                y: area.y + area.height - 1,
                width: area.width.saturating_sub(4),
                height: 1,
            };
            frame.render_widget(error_paragraph, error_area);
        }
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
        let dashboard_title = buffer
            .content()
            .iter()
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

    #[tokio::test]
    async fn test_dashboard_renders_table_not_paragraph() {
        // RED: This test should fail because we're currently using Paragraph
        // instead of the Table widget from widgets::dashboard
        let (op_tx, _op_rx) = unbounded_channel();
        let (_event_tx, event_rx) = unbounded_channel();

        let mut app = App::new(op_tx, event_rx);

        // Add some test processes
        use chrono::Utc;
        use pk_protocol::ProcessStatus;

        use std::sync::Arc;
        use tokio::sync::Notify;

        let process1 = Process {
            id: Uuid::new_v4(),
            pipeline_name: "test-pipeline-1".to_string(),
            status: ProcessStatus::Running,
            current_step_index: 0,
            logs: vec![],
            started_at: Utc::now(),
            completed_at: None,
            resume_notifier: Arc::new(Notify::new()),
        };

        let process2 = Process {
            id: Uuid::new_v4(),
            pipeline_name: "test-pipeline-2".to_string(),
            status: ProcessStatus::Completed,
            current_step_index: 5,
            logs: vec![],
            started_at: Utc::now(),
            completed_at: Some(Utc::now()),
            resume_notifier: Arc::new(Notify::new()),
        };

        app.processes.push(process1);
        app.processes.push(process2);

        let backend = TestBackend::new(120, 30);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                app.render(frame);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content = buffer
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();

        // The Table widget should render proper headers
        assert!(content.contains("ID"), "Should have 'ID' column header");
        assert!(
            content.contains("Pipeline"),
            "Should have 'Pipeline' column header"
        );
        assert!(
            content.contains("Status"),
            "Should have 'Status' column header"
        );
        assert!(content.contains("Step"), "Should have 'Step' column header");

        // Should contain process data
        assert!(
            content.contains("test-pipeline-1"),
            "Should show first pipeline name"
        );
        assert!(
            content.contains("test-pipeline-2"),
            "Should show second pipeline name"
        );

        // Should NOT contain the Paragraph-style ">" prefix
        // (Table uses ">>" as highlight symbol instead)
        // This assertion will fail with current Paragraph implementation
        let lines: Vec<&str> = content.split('\n').collect();
        let has_paragraph_style_prefix = lines
            .iter()
            .any(|line| line.trim_start().starts_with(">") && !line.trim_start().starts_with(">>"));
        assert!(
            !has_paragraph_style_prefix,
            "Should NOT use Paragraph-style single '>' prefix"
        );
    }
}
