//! Event handling utilities for the TUI.
//!
//! This module provides functions for handling different types of events:
//! - Core events (from pk-core)
//! - Keyboard events (user input)
//! - Command parsing and submission

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use pk_protocol::{Event, Op, Process, ProcessStatus};
use tokio::sync::mpsc::UnboundedSender;

/// Handle an event received from the core.
pub fn handle_core_event(processes: &mut Vec<Process>, event: Event) {
    match event {
        Event::ProcessStarted {
            process_id,
            pipeline_name,
        } => {
            let process = Process {
                id: process_id,
                pipeline_name,
                status: ProcessStatus::Running,
                current_step_index: 0,
                logs: Vec::new(),
                started_at: chrono::Utc::now(),
                completed_at: None,
            };
            processes.push(process);
        }
        Event::ProcessStatusUpdate {
            process_id,
            status,
            step_index,
        } => {
            if let Some(process) = processes.iter_mut().find(|p| p.id == process_id) {
                process.status = status;
                process.current_step_index = step_index;
            }
        }
        Event::ProcessLogChunk {
            process_id,
            content,
        } => {
            if let Some(process) = processes.iter_mut().find(|p| p.id == process_id) {
                process.logs.push(content);
            }
        }
        Event::ProcessCompleted { process_id } => {
            if let Some(process) = processes.iter_mut().find(|p| p.id == process_id) {
                process.status = ProcessStatus::Completed;
                process.completed_at = Some(chrono::Utc::now());
            }
        }
        Event::ProcessError { process_id, error } => {
            if let Some(process) = processes.iter_mut().find(|p| p.id == process_id) {
                process.status = ProcessStatus::Failed;
                process.logs.push(format!("ERROR: {}", error));
                process.completed_at = Some(chrono::Utc::now());
            }
        }
    }
}

/// Handle a keyboard event from the user.
///
/// Returns `true` if the application should exit, `false` otherwise.
pub fn handle_keyboard_event(
    key_event: KeyEvent,
    command_input: &mut String,
    selected_index: &mut usize,
    processes: &[Process],
    op_tx: &UnboundedSender<Op>,
) -> bool {
    if key_event.kind != KeyEventKind::Press {
        return false;
    }

    match key_event.code {
        KeyCode::Char('q') => {
            return true;
        }
        KeyCode::Up => {
            if *selected_index > 0 {
                *selected_index -= 1;
            }
        }
        KeyCode::Down => {
            if *selected_index < processes.len().saturating_sub(1) {
                *selected_index += 1;
            }
        }
        KeyCode::Char(c) => {
            command_input.push(c);
        }
        KeyCode::Backspace => {
            command_input.pop();
        }
        KeyCode::Enter => {
            submit_command(command_input, *selected_index, processes, op_tx);
        }
        _ => {}
    }

    false
}

/// Submit the current command input.
fn submit_command(
    command_input: &mut String,
    selected_index: usize,
    processes: &[Process],
    op_tx: &UnboundedSender<Op>,
) {
    if command_input.is_empty() {
        return;
    }

    // Parse simple slash commands
    if command_input.starts_with('/') {
        let parts: Vec<&str> = command_input.split_whitespace().collect();
        match parts.first().map(|s| *s) {
            Some("/start") => {
                if let Some(name) = parts.get(1) {
                    let _ = op_tx.send(Op::StartPipeline {
                        name: name.to_string(),
                        reference_file: None,
                    });
                }
            }
            Some("/pause") => {
                if let Some(process) = processes.get(selected_index) {
                    let _ = op_tx.send(Op::PauseProcess {
                        process_id: process.id,
                    });
                }
            }
            Some("/resume") => {
                if let Some(process) = processes.get(selected_index) {
                    let _ = op_tx.send(Op::ResumeProcess {
                        process_id: process.id,
                    });
                }
            }
            Some("/kill") => {
                if let Some(process) = processes.get(selected_index) {
                    let _ = op_tx.send(Op::KillProcess {
                        process_id: process.id,
                    });
                }
            }
            _ => {}
        }
    }

    command_input.clear();
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc::unbounded_channel;
    use uuid::Uuid;

    #[test]
    fn test_handle_core_event_process_started() {
        let mut processes = Vec::new();
        let process_id = Uuid::new_v4();

        handle_core_event(
            &mut processes,
            Event::ProcessStarted {
                process_id,
                pipeline_name: "test-pipeline".to_string(),
            },
        );

        assert_eq!(processes.len(), 1);
        assert_eq!(processes[0].id, process_id);
        assert_eq!(processes[0].pipeline_name, "test-pipeline");
    }

    #[test]
    fn test_handle_keyboard_event_quit() {
        let mut command_input = String::new();
        let mut selected_index = 0;
        let processes = Vec::new();
        let (op_tx, _op_rx) = unbounded_channel();

        let should_exit = handle_keyboard_event(
            KeyEvent::from(KeyCode::Char('q')),
            &mut command_input,
            &mut selected_index,
            &processes,
            &op_tx,
        );

        assert!(should_exit);
    }

    #[test]
    fn test_handle_keyboard_event_navigation() {
        let mut command_input = String::new();
        let mut selected_index = 1;
        let (op_tx, _op_rx) = unbounded_channel();

        // Create 3 test processes
        let processes = vec![
            Process {
                id: Uuid::new_v4(),
                pipeline_name: "p1".to_string(),
                status: ProcessStatus::Running,
                current_step_index: 0,
                logs: Vec::new(),
                started_at: chrono::Utc::now(),
                completed_at: None,
            },
            Process {
                id: Uuid::new_v4(),
                pipeline_name: "p2".to_string(),
                status: ProcessStatus::Running,
                current_step_index: 0,
                logs: Vec::new(),
                started_at: chrono::Utc::now(),
                completed_at: None,
            },
            Process {
                id: Uuid::new_v4(),
                pipeline_name: "p3".to_string(),
                status: ProcessStatus::Running,
                current_step_index: 0,
                logs: Vec::new(),
                started_at: chrono::Utc::now(),
                completed_at: None,
            },
        ];

        // Test Down
        handle_keyboard_event(
            KeyEvent::from(KeyCode::Down),
            &mut command_input,
            &mut selected_index,
            &processes,
            &op_tx,
        );
        assert_eq!(selected_index, 2);

        // Test Up
        handle_keyboard_event(
            KeyEvent::from(KeyCode::Up),
            &mut command_input,
            &mut selected_index,
            &processes,
            &op_tx,
        );
        assert_eq!(selected_index, 1);
    }

    #[test]
    fn test_command_input() {
        let mut command_input = String::new();
        let mut selected_index = 0;
        let processes = Vec::new();
        let (op_tx, _op_rx) = unbounded_channel();

        // Type some characters
        handle_keyboard_event(
            KeyEvent::from(KeyCode::Char('/')),
            &mut command_input,
            &mut selected_index,
            &processes,
            &op_tx,
        );
        handle_keyboard_event(
            KeyEvent::from(KeyCode::Char('s')),
            &mut command_input,
            &mut selected_index,
            &processes,
            &op_tx,
        );

        assert_eq!(command_input, "/s");

        // Backspace
        handle_keyboard_event(
            KeyEvent::from(KeyCode::Backspace),
            &mut command_input,
            &mut selected_index,
            &processes,
            &op_tx,
        );

        assert_eq!(command_input, "/");
    }
}
