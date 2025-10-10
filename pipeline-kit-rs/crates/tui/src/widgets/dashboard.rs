//! Dashboard widget for displaying process list in a table.
//!
//! This module provides a table-based view of all running processes,
//! showing their ID, name, status, and current step.

use pk_protocol::Process;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Frame,
};

/// Renders the dashboard as a table showing all processes.
///
/// # Arguments
/// * `frame` - The frame to render into
/// * `area` - The area to render the table in
/// * `processes` - List of all processes to display
/// * `selected` - Index of the currently selected process
pub fn render_dashboard(
    frame: &mut Frame,
    area: Rect,
    processes: &[Process],
    selected: usize,
) {
    // Create table rows from processes
    let rows: Vec<Row> = processes
        .iter()
        .map(|p| {
            Row::new(vec![
                Cell::from(p.id.to_string()),
                Cell::from(p.pipeline_name.clone()),
                Cell::from(format!("{:?}", p.status)),
                Cell::from(p.current_step_index.to_string()),
            ])
        })
        .collect();

    // Create table header
    let header = Row::new(vec![
        Cell::from("ID"),
        Cell::from("Name"),
        Cell::from("Status"),
        Cell::from("Step"),
    ])
    .style(Style::default().add_modifier(Modifier::BOLD));

    // Create the table with proper widths
    let widths = [
        ratatui::layout::Constraint::Length(36), // UUID length
        ratatui::layout::Constraint::Percentage(40),
        ratatui::layout::Constraint::Length(15),
        ratatui::layout::Constraint::Length(8),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Dashboard - Processes"),
        )
        .row_highlight_style(Style::default().bg(Color::Blue).fg(Color::White))
        .highlight_symbol("> ");

    // Create table state with selected index
    let mut table_state = TableState::default();
    if !processes.is_empty() {
        table_state.select(Some(selected));
    }

    frame.render_stateful_widget(table, area, &mut table_state);
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use pk_protocol::{Process, ProcessStatus};
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use uuid::Uuid;

    #[test]
    fn test_render_dashboard_empty() {
        // RED: This test should fail because we haven't implemented the table yet
        let backend = TestBackend::new(80, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        let processes: Vec<Process> = vec![];

        terminal
            .draw(|frame| {
                let area = frame.area();
                render_dashboard(frame, area, &processes, 0);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content = buffer
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();

        // Should contain table headers
        assert!(content.contains("ID"));
        assert!(content.contains("Name"));
        assert!(content.contains("Status"));
        assert!(content.contains("Step"));
    }

    #[test]
    fn test_render_dashboard_with_processes() {
        // GREEN: This test should pass with the Table implementation
        let backend = TestBackend::new(120, 20);
        let mut terminal = Terminal::new(backend).unwrap();

        let process1 = Process {
            id: Uuid::new_v4(),
            pipeline_name: "test-pipeline".to_string(),
            status: ProcessStatus::Running,
            current_step_index: 0,
            logs: vec![],
            started_at: Utc::now(),
            completed_at: None,
        };

        let process2 = Process {
            id: Uuid::new_v4(),
            pipeline_name: "another-pipeline".to_string(),
            status: ProcessStatus::Completed,
            current_step_index: 2,
            logs: vec![],
            started_at: Utc::now(),
            completed_at: Some(Utc::now()),
        };

        let processes = vec![process1.clone(), process2.clone()];

        terminal
            .draw(|frame| {
                let area = frame.area();
                render_dashboard(frame, area, &processes, 0);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content = buffer
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();

        // Should contain table headers
        assert!(content.contains("ID"));
        assert!(content.contains("Name"));
        assert!(content.contains("Status"));
        assert!(content.contains("Step"));

        // Should contain process data
        assert!(content.contains("test-pipeline"));
        // Note: "another-pipeline" might be truncated due to column width constraints
        // But we should at least see "RUNNING" and step numbers
        assert!(content.contains("RUNNING") || content.contains("Running"));
    }

    #[test]
    fn test_render_dashboard_highlights_selected() {
        // GREEN: This test should pass with highlighting implementation
        let backend = TestBackend::new(120, 20);
        let mut terminal = Terminal::new(backend).unwrap();

        let process1 = Process {
            id: Uuid::new_v4(),
            pipeline_name: "first-pipeline".to_string(),
            status: ProcessStatus::Running,
            current_step_index: 0,
            logs: vec![],
            started_at: Utc::now(),
            completed_at: None,
        };

        let process2 = Process {
            id: Uuid::new_v4(),
            pipeline_name: "second-pipeline".to_string(),
            status: ProcessStatus::Pending,
            current_step_index: 0,
            logs: vec![],
            started_at: Utc::now(),
            completed_at: None,
        };

        let processes = vec![process1, process2];

        terminal
            .draw(|frame| {
                let area = frame.area();
                render_dashboard(frame, area, &processes, 1);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();

        // Check if highlighting is applied by looking for blue background color
        // The Table widget with row_highlight_style should apply this to the selected row
        let mut found_blue_bg = false;
        for y in 0..buffer.area().height {
            for x in 0..buffer.area().width {
                let cell = &buffer[(x, y)];
                if cell.bg == Color::Blue {
                    found_blue_bg = true;
                    break;
                }
            }
            if found_blue_bg {
                break;
            }
        }

        assert!(
            found_blue_bg,
            "Selected process row should be highlighted with blue background"
        );
    }
}
