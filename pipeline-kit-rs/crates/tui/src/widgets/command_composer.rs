//! Command composer widget with slash command autocomplete.
//!
//! This widget provides a text input field for entering commands, with
//! autocomplete suggestions when the user types a slash command.

use crate::event::EventStatus;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use pk_protocol::Op;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Widget;
use uuid::Uuid;

/// Available slash commands with their descriptions.
const COMMANDS: &[(&str, &str)] = &[
    ("/start <pipeline>", "Start a new pipeline"),
    ("/pause <process_id>", "Pause a running process"),
    ("/resume <process_id>", "Resume a paused process"),
    ("/kill <process_id>", "Kill a process"),
    ("/list", "List all processes"),
];

/// Command composer state.
#[derive(Debug, Clone)]
pub struct CommandComposer {
    /// Current input text
    input: String,
    /// Current cursor position
    cursor_pos: usize,
    /// Whether autocomplete popup should be shown
    show_popup: bool,
    /// Selected index in the autocomplete list
    selected_index: usize,
}

impl Default for CommandComposer {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandComposer {
    /// Create a new command composer.
    pub fn new() -> Self {
        Self {
            input: String::new(),
            cursor_pos: 0,
            show_popup: false,
            selected_index: 0,
        }
    }

    /// Get the current input text.
    pub fn input(&self) -> &str {
        &self.input
    }

    /// Check if autocomplete popup should be shown.
    pub fn should_show_popup(&self) -> bool {
        self.show_popup
    }

    /// Get filtered command suggestions based on current input.
    pub fn suggestions(&self) -> Vec<(&'static str, &'static str)> {
        if !self.input.starts_with('/') {
            return Vec::new();
        }

        let filter = self.input.trim();
        if filter == "/" {
            // Show all commands
            return COMMANDS.to_vec();
        }

        // Simple prefix matching for now
        COMMANDS
            .iter()
            .filter(|(cmd, _)| cmd.starts_with(filter))
            .copied()
            .collect()
    }

    /// Get the currently selected suggestion.
    pub fn selected_suggestion(&self) -> Option<(&'static str, &'static str)> {
        let suggestions = self.suggestions();
        suggestions.get(self.selected_index).copied()
    }

    /// Insert a character at the cursor position.
    pub fn insert_char(&mut self, c: char) {
        self.input.insert(self.cursor_pos, c);
        self.cursor_pos += 1;
        self.update_popup_state();
    }

    /// Delete the character before the cursor (backspace).
    pub fn delete_char(&mut self) {
        if self.cursor_pos > 0 {
            self.input.remove(self.cursor_pos - 1);
            self.cursor_pos -= 1;
            self.update_popup_state();
        }
    }

    /// Clear all input.
    pub fn clear(&mut self) {
        self.input.clear();
        self.cursor_pos = 0;
        self.show_popup = false;
        self.selected_index = 0;
    }

    /// Move cursor left.
    pub fn move_cursor_left(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
        }
    }

    /// Move cursor right.
    pub fn move_cursor_right(&mut self) {
        if self.cursor_pos < self.input.len() {
            self.cursor_pos += 1;
        }
    }

    /// Move selection up in autocomplete popup.
    pub fn move_selection_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// Move selection down in autocomplete popup.
    pub fn move_selection_down(&mut self) {
        let suggestions = self.suggestions();
        if self.selected_index + 1 < suggestions.len() {
            self.selected_index += 1;
        }
    }

    /// Complete with the currently selected suggestion (Tab key).
    pub fn complete_with_selection(&mut self) {
        if let Some((cmd, _)) = self.selected_suggestion() {
            // Extract just the command name (without arguments placeholder)
            let cmd_name = cmd.split_whitespace().next().unwrap_or(cmd);
            self.input = format!("{} ", cmd_name);
            self.cursor_pos = self.input.len();
            self.show_popup = false;
            self.selected_index = 0;
        }
    }

    /// Update the popup state based on current input.
    fn update_popup_state(&mut self) {
        self.show_popup = self.input.starts_with('/') && !self.input.ends_with(' ');

        // Reset selection if needed
        let suggestions = self.suggestions();
        if self.selected_index >= suggestions.len() {
            self.selected_index = suggestions.len().saturating_sub(1);
        }
    }

    /// Render the input field.
    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Command (q to quit)");

        let inner = block.inner(area);
        block.render(area, buf);

        let text = format!("> {}", self.input);
        let paragraph = Paragraph::new(text).style(Style::default().fg(Color::Yellow));
        paragraph.render(inner, buf);
    }

    /// Render the autocomplete popup.
    pub fn render_popup(&self, area: Rect, buf: &mut Buffer) {
        if !self.show_popup {
            return;
        }

        let suggestions = self.suggestions();
        if suggestions.is_empty() {
            return;
        }

        let block = Block::default()
            .borders(Borders::ALL)
            .title("Suggestions")
            .style(Style::default().bg(Color::Black));

        let inner = block.inner(area);
        block.render(area, buf);

        // Render suggestions
        let mut y = inner.y;
        for (i, (cmd, desc)) in suggestions.iter().enumerate() {
            if y >= inner.y + inner.height {
                break;
            }

            let style = if i == self.selected_index {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let line = Line::from(vec![
                Span::styled(format!("{:<25}", cmd), style),
                Span::styled(desc.to_string(), style.fg(Color::Gray)),
            ]);

            buf.set_line(inner.x, y, &line, inner.width);
            y += 1;
        }
    }

    /// Handle a key event and return whether it was consumed.
    ///
    /// This method implements the widget's event handling logic, allowing the
    /// CommandComposer to handle its own keyboard input. It returns `EventStatus::Consumed`
    /// if the event was handled, or `EventStatus::NotConsumed` if it should be passed
    /// to the next handler in the chain of responsibility.
    ///
    /// Events that are NOT consumed (passed to parent handler):
    /// - Enter key (command submission handled by App)
    /// - 'q' when input is empty (global quit)
    /// - Ctrl+C (global quit)
    /// - Up/Down when popup is not shown (process navigation)
    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> EventStatus {
        // Ignore key release events
        if key_event.kind != KeyEventKind::Press {
            return EventStatus::Consumed;
        }

        match key_event.code {
            KeyCode::Up | KeyCode::Down => self.handle_vertical_navigation(key_event.code),
            KeyCode::Tab => self.handle_tab(),
            KeyCode::Char(c) => self.handle_char_input(c),
            KeyCode::Backspace => self.handle_backspace(),
            KeyCode::Left | KeyCode::Right => self.handle_cursor_move(key_event.code),
            KeyCode::Esc => self.handle_escape(),
            KeyCode::Enter => EventStatus::NotConsumed,
            _ => EventStatus::NotConsumed,
        }
    }

    /// Handle vertical navigation (Up/Down keys).
    ///
    /// Only consumes the event if the autocomplete popup is shown.
    /// Otherwise, returns NotConsumed to allow App to handle process navigation.
    fn handle_vertical_navigation(&mut self, code: KeyCode) -> EventStatus {
        if !self.should_show_popup() {
            // Not consumed - let App handle process navigation
            return EventStatus::NotConsumed;
        }

        match code {
            KeyCode::Up => self.move_selection_up(),
            KeyCode::Down => self.move_selection_down(),
            _ => unreachable!(),
        }

        EventStatus::Consumed
    }

    /// Handle Tab key for autocomplete.
    ///
    /// Only consumes the event if the autocomplete popup is shown.
    fn handle_tab(&mut self) -> EventStatus {
        if !self.should_show_popup() {
            return EventStatus::NotConsumed;
        }

        self.complete_with_selection();
        EventStatus::Consumed
    }

    /// Handle character input.
    ///
    /// Special case: 'q' when input is empty is not consumed (global quit).
    fn handle_char_input(&mut self, c: char) -> EventStatus {
        // Don't consume 'q' when input is empty (it's a global quit)
        if c == 'q' && self.input.is_empty() {
            return EventStatus::NotConsumed;
        }

        self.insert_char(c);
        EventStatus::Consumed
    }

    /// Handle backspace key.
    fn handle_backspace(&mut self) -> EventStatus {
        self.delete_char();
        EventStatus::Consumed
    }

    /// Handle cursor movement (Left/Right keys).
    fn handle_cursor_move(&mut self, code: KeyCode) -> EventStatus {
        match code {
            KeyCode::Left => self.move_cursor_left(),
            KeyCode::Right => self.move_cursor_right(),
            _ => unreachable!(),
        }

        EventStatus::Consumed
    }

    /// Handle Escape key.
    fn handle_escape(&mut self) -> EventStatus {
        self.clear();
        EventStatus::Consumed
    }

    /// Parse the current input and generate an Op if valid.
    ///
    /// Returns Ok(Some(Op)) if a valid command was parsed,
    /// Ok(None) if input is empty or whitespace,
    /// Err(String) if the command is invalid.
    pub fn parse_command(&self) -> Result<Option<Op>, String> {
        let input = self.input.trim();

        if input.is_empty() {
            return Ok(None);
        }

        // Parse slash commands
        if input.starts_with('/') {
            let parts: Vec<&str> = input.split_whitespace().collect();
            let cmd = parts.first().ok_or("Empty command")?;

            match *cmd {
                "/start" => {
                    let pipeline_name = parts.get(1).ok_or("Missing pipeline name")?;
                    Ok(Some(Op::StartPipeline {
                        name: pipeline_name.to_string(),
                        reference_file: None,
                    }))
                }
                "/pause" => {
                    let process_id_str = parts.get(1).ok_or("Missing process ID")?;
                    let process_id =
                        Uuid::parse_str(process_id_str).map_err(|_| "Invalid process ID format")?;
                    Ok(Some(Op::PauseProcess { process_id }))
                }
                "/resume" => {
                    let process_id_str = parts.get(1).ok_or("Missing process ID")?;
                    let process_id =
                        Uuid::parse_str(process_id_str).map_err(|_| "Invalid process ID format")?;
                    Ok(Some(Op::ResumeProcess { process_id }))
                }
                "/kill" => {
                    let process_id_str = parts.get(1).ok_or("Missing process ID")?;
                    let process_id =
                        Uuid::parse_str(process_id_str).map_err(|_| "Invalid process ID format")?;
                    Ok(Some(Op::KillProcess { process_id }))
                }
                "/list" => Ok(Some(Op::GetDashboardState)),
                _ => Err(format!("Unknown command: {}", cmd)),
            }
        } else {
            // Non-slash commands could be handled here in the future
            Err("Invalid command. Commands must start with '/'".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::EventStatus;
    use crossterm::event::{KeyCode, KeyEvent};

    #[test]
    fn test_new_composer_is_empty() {
        let composer = CommandComposer::new();
        assert_eq!(composer.input(), "");
        assert!(!composer.should_show_popup());
    }

    #[test]
    fn test_typing_slash_shows_popup() {
        let mut composer = CommandComposer::new();
        composer.insert_char('/');

        assert_eq!(composer.input(), "/");
        assert!(composer.should_show_popup());

        let suggestions = composer.suggestions();
        assert_eq!(suggestions.len(), COMMANDS.len());
    }

    #[test]
    fn test_typing_start_filters_suggestions() {
        let mut composer = CommandComposer::new();
        composer.insert_char('/');
        composer.insert_char('s');
        composer.insert_char('t');
        composer.insert_char('a');
        composer.insert_char('r');
        composer.insert_char('t');

        assert_eq!(composer.input(), "/start");
        assert!(composer.should_show_popup());

        let suggestions = composer.suggestions();
        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].0, "/start <pipeline>");
    }

    #[test]
    fn test_no_suggestions_for_non_slash_input() {
        let mut composer = CommandComposer::new();
        composer.insert_char('h');
        composer.insert_char('e');
        composer.insert_char('l');
        composer.insert_char('l');
        composer.insert_char('o');

        assert_eq!(composer.input(), "hello");
        assert!(!composer.should_show_popup());
        assert!(composer.suggestions().is_empty());
    }

    #[test]
    fn test_backspace_removes_character() {
        let mut composer = CommandComposer::new();
        composer.insert_char('/');
        composer.insert_char('s');
        composer.insert_char('t');

        assert_eq!(composer.input(), "/st");

        composer.delete_char();
        assert_eq!(composer.input(), "/s");

        composer.delete_char();
        assert_eq!(composer.input(), "/");

        composer.delete_char();
        assert_eq!(composer.input(), "");
    }

    #[test]
    fn test_clear_resets_state() {
        let mut composer = CommandComposer::new();
        composer.insert_char('/');
        composer.insert_char('s');
        composer.insert_char('t');

        composer.clear();

        assert_eq!(composer.input(), "");
        assert!(!composer.should_show_popup());
        assert_eq!(composer.cursor_pos, 0);
    }

    #[test]
    fn test_selection_navigation() {
        let mut composer = CommandComposer::new();
        composer.insert_char('/');

        // Should start at index 0
        assert_eq!(composer.selected_index, 0);

        composer.move_selection_down();
        assert_eq!(composer.selected_index, 1);

        composer.move_selection_down();
        assert_eq!(composer.selected_index, 2);

        composer.move_selection_up();
        assert_eq!(composer.selected_index, 1);

        composer.move_selection_up();
        assert_eq!(composer.selected_index, 0);

        // Should not go below 0
        composer.move_selection_up();
        assert_eq!(composer.selected_index, 0);
    }

    #[test]
    fn test_tab_completion() {
        let mut composer = CommandComposer::new();
        composer.insert_char('/');
        composer.insert_char('s');
        composer.insert_char('t');

        // Should suggest /start <pipeline>
        let suggestions = composer.suggestions();
        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].0, "/start <pipeline>");

        // Tab completion should fill in the command
        composer.complete_with_selection();

        assert_eq!(composer.input(), "/start ");
        assert!(!composer.should_show_popup());
    }

    #[test]
    fn test_selected_suggestion() {
        let mut composer = CommandComposer::new();
        composer.insert_char('/');

        let selected = composer.selected_suggestion();
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().0, "/start <pipeline>");

        composer.move_selection_down();
        let selected = composer.selected_suggestion();
        assert_eq!(selected.unwrap().0, "/pause <process_id>");
    }

    #[test]
    fn test_popup_hides_after_space() {
        let mut composer = CommandComposer::new();
        composer.insert_char('/');
        composer.insert_char('s');
        composer.insert_char('t');
        composer.insert_char('a');
        composer.insert_char('r');
        composer.insert_char('t');

        assert!(composer.should_show_popup());

        composer.insert_char(' ');

        // Popup should hide after typing space (entering arguments)
        assert!(!composer.should_show_popup());
    }

    #[test]
    fn test_parse_start_command() {
        let mut composer = CommandComposer::new();
        for c in "/start my-pipeline".chars() {
            composer.insert_char(c);
        }

        let result = composer.parse_command();
        assert!(result.is_ok());

        let op = result.unwrap();
        assert!(op.is_some());

        match op.unwrap() {
            Op::StartPipeline {
                name,
                reference_file,
            } => {
                assert_eq!(name, "my-pipeline");
                assert!(reference_file.is_none());
            }
            _ => panic!("Expected StartPipeline op"),
        }
    }

    #[test]
    fn test_parse_list_command() {
        let mut composer = CommandComposer::new();
        for c in "/list".chars() {
            composer.insert_char(c);
        }

        let result = composer.parse_command();
        assert!(result.is_ok());

        let op = result.unwrap();
        assert!(op.is_some());

        match op.unwrap() {
            Op::GetDashboardState => {}
            _ => panic!("Expected GetDashboardState op"),
        }
    }

    #[test]
    fn test_parse_pause_command() {
        let process_id = Uuid::new_v4();
        let mut composer = CommandComposer::new();
        for c in format!("/pause {}", process_id).chars() {
            composer.insert_char(c);
        }

        let result = composer.parse_command();
        assert!(result.is_ok());

        let op = result.unwrap();
        assert!(op.is_some());

        match op.unwrap() {
            Op::PauseProcess { process_id: id } => {
                assert_eq!(id, process_id);
            }
            _ => panic!("Expected PauseProcess op"),
        }
    }

    #[test]
    fn test_parse_resume_command() {
        let process_id = Uuid::new_v4();
        let mut composer = CommandComposer::new();
        for c in format!("/resume {}", process_id).chars() {
            composer.insert_char(c);
        }

        let result = composer.parse_command();
        assert!(result.is_ok());

        let op = result.unwrap();
        assert!(op.is_some());

        match op.unwrap() {
            Op::ResumeProcess { process_id: id } => {
                assert_eq!(id, process_id);
            }
            _ => panic!("Expected ResumeProcess op"),
        }
    }

    #[test]
    fn test_parse_kill_command() {
        let process_id = Uuid::new_v4();
        let mut composer = CommandComposer::new();
        for c in format!("/kill {}", process_id).chars() {
            composer.insert_char(c);
        }

        let result = composer.parse_command();
        assert!(result.is_ok());

        let op = result.unwrap();
        assert!(op.is_some());

        match op.unwrap() {
            Op::KillProcess { process_id: id } => {
                assert_eq!(id, process_id);
            }
            _ => panic!("Expected KillProcess op"),
        }
    }

    #[test]
    fn test_parse_empty_command() {
        let composer = CommandComposer::new();

        let result = composer.parse_command();
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_parse_invalid_command() {
        let mut composer = CommandComposer::new();
        for c in "/invalid".chars() {
            composer.insert_char(c);
        }

        let result = composer.parse_command();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown command"));
    }

    #[test]
    fn test_parse_missing_argument() {
        let mut composer = CommandComposer::new();
        for c in "/start".chars() {
            composer.insert_char(c);
        }

        let result = composer.parse_command();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing pipeline name"));
    }

    #[test]
    fn test_parse_invalid_uuid() {
        let mut composer = CommandComposer::new();
        for c in "/pause invalid-uuid".chars() {
            composer.insert_char(c);
        }

        let result = composer.parse_command();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid process ID format"));
    }

    #[test]
    fn test_parse_non_slash_command() {
        let mut composer = CommandComposer::new();
        for c in "hello world".chars() {
            composer.insert_char(c);
        }

        let result = composer.parse_command();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Commands must start with"));
    }

    // ========================================================================
    // RED TESTS: Event Handling Delegation (Ticket 9.3)
    // ========================================================================

    #[test]
    fn test_handle_key_event_cursor_left() {
        // RED: This test will fail because handle_key_event doesn't exist yet
        let mut composer = CommandComposer::new();
        composer.insert_char('/');
        composer.insert_char('s');
        composer.insert_char('t');
        assert_eq!(composer.cursor_pos, 3);

        let status = composer.handle_key_event(KeyEvent::from(KeyCode::Left));
        assert_eq!(status, EventStatus::Consumed);
        assert_eq!(composer.cursor_pos, 2);
    }

    #[test]
    fn test_handle_key_event_cursor_right() {
        // RED: This test will fail because handle_key_event doesn't exist yet
        let mut composer = CommandComposer::new();
        composer.insert_char('/');
        composer.insert_char('s');
        composer.move_cursor_left();
        assert_eq!(composer.cursor_pos, 1);

        let status = composer.handle_key_event(KeyEvent::from(KeyCode::Right));
        assert_eq!(status, EventStatus::Consumed);
        assert_eq!(composer.cursor_pos, 2);
    }

    #[test]
    fn test_handle_key_event_char_input() {
        // RED: This test will fail because handle_key_event doesn't exist yet
        let mut composer = CommandComposer::new();

        let status = composer.handle_key_event(KeyEvent::from(KeyCode::Char('/')));
        assert_eq!(status, EventStatus::Consumed);
        assert_eq!(composer.input(), "/");

        let status = composer.handle_key_event(KeyEvent::from(KeyCode::Char('s')));
        assert_eq!(status, EventStatus::Consumed);
        assert_eq!(composer.input(), "/s");
    }

    #[test]
    fn test_handle_key_event_backspace() {
        // RED: This test will fail because handle_key_event doesn't exist yet
        let mut composer = CommandComposer::new();
        composer.insert_char('/');
        composer.insert_char('s');

        let status = composer.handle_key_event(KeyEvent::from(KeyCode::Backspace));
        assert_eq!(status, EventStatus::Consumed);
        assert_eq!(composer.input(), "/");
    }

    #[test]
    fn test_handle_key_event_up_with_popup() {
        // RED: This test will fail because handle_key_event doesn't exist yet
        let mut composer = CommandComposer::new();
        composer.insert_char('/');
        assert!(composer.should_show_popup());
        assert_eq!(composer.selected_index, 0);

        let status = composer.handle_key_event(KeyEvent::from(KeyCode::Down));
        assert_eq!(status, EventStatus::Consumed);
        assert_eq!(composer.selected_index, 1);

        let status = composer.handle_key_event(KeyEvent::from(KeyCode::Up));
        assert_eq!(status, EventStatus::Consumed);
        assert_eq!(composer.selected_index, 0);
    }

    #[test]
    fn test_handle_key_event_down_with_popup() {
        // RED: This test will fail because handle_key_event doesn't exist yet
        let mut composer = CommandComposer::new();
        composer.insert_char('/');
        assert!(composer.should_show_popup());
        assert_eq!(composer.selected_index, 0);

        let status = composer.handle_key_event(KeyEvent::from(KeyCode::Down));
        assert_eq!(status, EventStatus::Consumed);
        assert_eq!(composer.selected_index, 1);
    }

    #[test]
    fn test_handle_key_event_tab_completes_command() {
        // RED: This test will fail because handle_key_event doesn't exist yet
        let mut composer = CommandComposer::new();
        composer.insert_char('/');
        composer.insert_char('s');
        composer.insert_char('t');
        assert!(composer.should_show_popup());

        let status = composer.handle_key_event(KeyEvent::from(KeyCode::Tab));
        assert_eq!(status, EventStatus::Consumed);
        assert_eq!(composer.input(), "/start ");
        assert!(!composer.should_show_popup());
    }

    #[test]
    fn test_handle_key_event_esc_clears_input() {
        // RED: This test will fail because handle_key_event doesn't exist yet
        let mut composer = CommandComposer::new();
        composer.insert_char('/');
        composer.insert_char('s');
        composer.insert_char('t');

        let status = composer.handle_key_event(KeyEvent::from(KeyCode::Esc));
        assert_eq!(status, EventStatus::Consumed);
        assert_eq!(composer.input(), "");
    }

    #[test]
    fn test_handle_key_event_enter_not_consumed() {
        // RED: Enter should NOT be consumed by CommandComposer - it's handled by App
        let mut composer = CommandComposer::new();
        composer.insert_char('/');
        composer.insert_char('l');
        composer.insert_char('i');
        composer.insert_char('s');
        composer.insert_char('t');

        let status = composer.handle_key_event(KeyEvent::from(KeyCode::Enter));
        assert_eq!(status, EventStatus::NotConsumed);
        // Input should remain unchanged
        assert_eq!(composer.input(), "/list");
    }

    #[test]
    fn test_handle_key_event_quit_keys_not_consumed() {
        // When input is empty, 'q' should NOT be consumed (it's a global quit)
        let mut composer = CommandComposer::new();

        let status = composer.handle_key_event(KeyEvent::from(KeyCode::Char('q')));
        assert_eq!(status, EventStatus::NotConsumed);
        // Input should remain empty
        assert_eq!(composer.input(), "");
    }

    #[test]
    fn test_handle_key_event_q_consumed_with_input() {
        // But if there's already input, 'q' should be treated as a regular character
        let mut composer = CommandComposer::new();
        composer.insert_char('/');

        let status = composer.handle_key_event(KeyEvent::from(KeyCode::Char('q')));
        assert_eq!(status, EventStatus::Consumed);
        assert_eq!(composer.input(), "/q");
    }

    #[test]
    fn test_handle_key_event_up_down_without_popup_not_consumed() {
        // RED: When popup is not shown, Up/Down should NOT be consumed
        // (they're for process navigation in App)
        let mut composer = CommandComposer::new();
        composer.insert_char('h');
        composer.insert_char('e');
        composer.insert_char('l');
        composer.insert_char('l');
        composer.insert_char('o');
        assert!(!composer.should_show_popup());

        let status = composer.handle_key_event(KeyEvent::from(KeyCode::Up));
        assert_eq!(status, EventStatus::NotConsumed);

        let status = composer.handle_key_event(KeyEvent::from(KeyCode::Down));
        assert_eq!(status, EventStatus::NotConsumed);
    }
}
