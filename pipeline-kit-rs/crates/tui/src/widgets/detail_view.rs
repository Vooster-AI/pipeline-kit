//! Detail view widget for displaying process logs with scrolling support.
//!
//! This widget displays the logs and details of a selected process in a scrollable view.
//! It supports keyboard navigation (j/k, PageUp/PageDown) and shows a scrollbar to indicate position.

use pk_protocol::Process;
use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};

/// Widget for displaying process details with scrolling support.
pub struct DetailView {
    /// Current scroll offset (number of lines scrolled from the top).
    pub scroll_offset: usize,
}

impl DetailView {
    /// Create a new DetailView with scroll offset at the top.
    pub fn new() -> Self {
        Self { scroll_offset: 0 }
    }

    /// Render the detail view for a given process.
    ///
    /// # Arguments
    ///
    /// * `frame` - The ratatui frame to render to
    /// * `area` - The area to render within
    /// * `process` - Optional reference to the process to display
    pub fn render(&self, frame: &mut Frame, area: Rect, process: Option<&Process>) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Detail - Process Logs");

        let text = if let Some(process) = process {
            if process.logs.is_empty() {
                "No logs yet.".to_string()
            } else {
                process.logs.join("\n")
            }
        } else {
            "No process selected.".to_string()
        };

        let paragraph = Paragraph::new(text)
            .block(block)
            .scroll((self.scroll_offset as u16, 0));

        frame.render_widget(paragraph, area);

        // Render scrollbar if there is content to scroll
        if let Some(process) = process {
            if !process.logs.is_empty() {
                let total_lines = process.logs.len();
                let visible_lines = area.height.saturating_sub(2) as usize; // Subtract 2 for borders

                // Only show scrollbar if content exceeds visible area
                if total_lines > visible_lines {
                    let mut scrollbar_state = ScrollbarState::default()
                        .content_length(total_lines)
                        .viewport_content_length(visible_lines)
                        .position(self.scroll_offset);

                    let scrollbar = Scrollbar::default()
                        .orientation(ScrollbarOrientation::VerticalRight)
                        .begin_symbol(Some("↑"))
                        .end_symbol(Some("↓"));

                    frame.render_stateful_widget(scrollbar, area, &mut scrollbar_state);
                }
            }
        }
    }

    /// Scroll up by one line.
    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    /// Scroll down by one line.
    ///
    /// # Arguments
    ///
    /// * `max` - The maximum scroll offset (typically total_lines - visible_lines)
    pub fn scroll_down(&mut self, max: usize) {
        self.scroll_offset = (self.scroll_offset + 1).min(max);
    }

    /// Scroll up by a page (viewport height).
    ///
    /// # Arguments
    ///
    /// * `page_size` - Number of lines in a page
    pub fn page_up(&mut self, page_size: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(page_size);
    }

    /// Scroll down by a page (viewport height).
    ///
    /// # Arguments
    ///
    /// * `page_size` - Number of lines in a page
    /// * `max` - The maximum scroll offset
    pub fn page_down(&mut self, page_size: usize, max: usize) {
        self.scroll_offset = (self.scroll_offset + page_size).min(max);
    }

    /// Reset scroll to the top.
    pub fn scroll_to_top(&mut self) {
        self.scroll_offset = 0;
    }

    /// Scroll to the bottom.
    ///
    /// # Arguments
    ///
    /// * `max` - The maximum scroll offset
    pub fn scroll_to_bottom(&mut self, max: usize) {
        self.scroll_offset = max;
    }
}

impl Default for DetailView {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pk_protocol::ProcessStatus;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use uuid::Uuid;

    fn create_test_process(logs: Vec<String>) -> Process {
        Process {
            id: Uuid::new_v4(),
            pipeline_name: "test-pipeline".to_string(),
            status: ProcessStatus::Running,
            current_step_index: 0,
            started_at: chrono::Utc::now(),
            completed_at: None,
            logs,
        }
    }

    #[test]
    fn test_detail_view_renders_empty_state() {
        let detail_view = DetailView::new();
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                detail_view.render(frame, frame.area(), None);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content = buffer
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();

        assert!(content.contains("No process selected"));
        assert!(content.contains("Detail - Process Logs"));
    }

    #[test]
    fn test_detail_view_renders_process_logs() {
        let detail_view = DetailView::new();
        let process = create_test_process(vec![
            "Log line 1".to_string(),
            "Log line 2".to_string(),
            "Log line 3".to_string(),
        ]);

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                detail_view.render(frame, frame.area(), Some(&process));
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content = buffer
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();

        assert!(content.contains("Log line 1"));
        assert!(content.contains("Log line 2"));
        assert!(content.contains("Log line 3"));
    }

    #[test]
    fn test_detail_view_scroll_up() {
        let mut detail_view = DetailView::new();
        detail_view.scroll_offset = 5;

        detail_view.scroll_up();
        assert_eq!(detail_view.scroll_offset, 4);

        detail_view.scroll_up();
        assert_eq!(detail_view.scroll_offset, 3);
    }

    #[test]
    fn test_detail_view_scroll_up_at_top() {
        let mut detail_view = DetailView::new();
        detail_view.scroll_offset = 0;

        detail_view.scroll_up();
        assert_eq!(detail_view.scroll_offset, 0); // Should not go below 0
    }

    #[test]
    fn test_detail_view_scroll_down() {
        let mut detail_view = DetailView::new();
        detail_view.scroll_offset = 0;

        detail_view.scroll_down(10);
        assert_eq!(detail_view.scroll_offset, 1);

        detail_view.scroll_down(10);
        assert_eq!(detail_view.scroll_offset, 2);
    }

    #[test]
    fn test_detail_view_scroll_down_at_max() {
        let mut detail_view = DetailView::new();
        detail_view.scroll_offset = 10;

        detail_view.scroll_down(10);
        assert_eq!(detail_view.scroll_offset, 10); // Should not exceed max
    }

    #[test]
    fn test_detail_view_page_up() {
        let mut detail_view = DetailView::new();
        detail_view.scroll_offset = 20;

        detail_view.page_up(10);
        assert_eq!(detail_view.scroll_offset, 10);

        detail_view.page_up(10);
        assert_eq!(detail_view.scroll_offset, 0);
    }

    #[test]
    fn test_detail_view_page_down() {
        let mut detail_view = DetailView::new();
        detail_view.scroll_offset = 0;

        detail_view.page_down(10, 50);
        assert_eq!(detail_view.scroll_offset, 10);

        detail_view.page_down(10, 50);
        assert_eq!(detail_view.scroll_offset, 20);
    }

    #[test]
    fn test_detail_view_scroll_to_top() {
        let mut detail_view = DetailView::new();
        detail_view.scroll_offset = 42;

        detail_view.scroll_to_top();
        assert_eq!(detail_view.scroll_offset, 0);
    }

    #[test]
    fn test_detail_view_scroll_to_bottom() {
        let mut detail_view = DetailView::new();
        detail_view.scroll_offset = 0;

        detail_view.scroll_to_bottom(100);
        assert_eq!(detail_view.scroll_offset, 100);
    }

    #[test]
    fn test_detail_view_renders_with_scroll_offset() {
        let mut detail_view = DetailView::new();
        detail_view.scroll_offset = 2;

        let process = create_test_process(vec![
            "Line 0".to_string(),
            "Line 1".to_string(),
            "Line 2".to_string(),
            "Line 3".to_string(),
            "Line 4".to_string(),
        ]);

        let backend = TestBackend::new(80, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                detail_view.render(frame, frame.area(), Some(&process));
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content = buffer
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();

        // With scroll_offset=2, we should see Line 2, 3, 4 but not Line 0, 1
        assert!(content.contains("Line 2"));
        assert!(content.contains("Line 3"));
        assert!(content.contains("Line 4"));
    }
}
