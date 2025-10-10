//! Terminal UI initialization and event handling.
//!
//! This module provides the `Tui` wrapper around ratatui's Terminal,
//! handling raw mode setup, event streaming, and frame scheduling.

use anyhow::Result;
use crossterm::event::DisableBracketedPaste;
use crossterm::event::EnableBracketedPaste;
use crossterm::event::Event;
use crossterm::event::KeyEvent;
use crossterm::execute;
use crossterm::terminal::disable_raw_mode;
use crossterm::terminal::enable_raw_mode;
use crossterm::terminal::EnterAlternateScreen;
use crossterm::terminal::LeaveAlternateScreen;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::stdout;
use std::io::Stdout;
use std::pin::Pin;
use std::time::Duration;
use std::time::Instant;
use tokio::select;
use tokio_stream::Stream;
use tokio_stream::StreamExt;

/// Type alias for the terminal backend we're using.
pub type TerminalBackend = CrosstermBackend<Stdout>;

/// TUI events that can be emitted.
#[derive(Debug)]
pub enum TuiEvent {
    /// Keyboard event.
    Key(KeyEvent),
    /// Paste event (from bracketed paste).
    Paste(String),
    /// Draw event (triggered by frame scheduler).
    Draw,
}

/// Main TUI wrapper.
pub struct Tui {
    /// The underlying ratatui terminal.
    terminal: Terminal<TerminalBackend>,
    /// Channel for scheduling frames.
    frame_schedule_tx: tokio::sync::mpsc::UnboundedSender<Instant>,
    /// Broadcast channel for draw events.
    draw_tx: tokio::sync::broadcast::Sender<()>,
}

impl Tui {
    /// Initialize the terminal in raw mode.
    pub fn init() -> Result<Self> {
        // Enable raw mode and bracketed paste
        enable_raw_mode()?;
        execute!(stdout(), EnableBracketedPaste)?;
        execute!(stdout(), EnterAlternateScreen)?;

        // Set panic hook to restore terminal on panic
        set_panic_hook();

        // Create terminal backend
        let backend = CrosstermBackend::new(stdout());
        let terminal = Terminal::new(backend)?;

        // Create frame scheduling channels
        let (frame_schedule_tx, frame_schedule_rx) = tokio::sync::mpsc::unbounded_channel();
        let (draw_tx, _) = tokio::sync::broadcast::channel(1);

        // Spawn background task to coalesce frame requests
        let draw_tx_clone = draw_tx.clone();
        tokio::spawn(async move {
            use tokio::time::sleep_until;
            use tokio::time::Instant as TokioInstant;

            let mut rx = frame_schedule_rx;
            let mut next_deadline: Option<Instant> = None;

            loop {
                let target =
                    next_deadline.unwrap_or_else(|| Instant::now() + Duration::from_secs(3600));
                let sleep_fut = sleep_until(TokioInstant::from_std(target));
                tokio::pin!(sleep_fut);

                select! {
                    recv = rx.recv() => {
                        match recv {
                            Some(at) => {
                                if next_deadline.is_none() || at < next_deadline.unwrap() {
                                    next_deadline = Some(at);
                                }
                                continue;
                            }
                            None => break,
                        }
                    }
                    _ = &mut sleep_fut => {
                        if next_deadline.is_some() {
                            next_deadline = None;
                            let _ = draw_tx_clone.send(());
                        }
                    }
                }
            }
        });

        Ok(Self {
            terminal,
            frame_schedule_tx,
            draw_tx,
        })
    }

    /// Restore the terminal to its original state.
    pub fn restore(&mut self) -> Result<()> {
        disable_raw_mode()?;
        execute!(stdout(), DisableBracketedPaste)?;
        execute!(stdout(), LeaveAlternateScreen)?;
        Ok(())
    }

    /// Get a frame requester for scheduling draws.
    pub fn frame_requester(&self) -> FrameRequester {
        FrameRequester {
            frame_schedule_tx: self.frame_schedule_tx.clone(),
        }
    }

    /// Create an event stream for TUI events.
    pub fn event_stream(&self) -> Pin<Box<dyn Stream<Item = TuiEvent> + Send + 'static>> {
        let mut crossterm_events = crossterm::event::EventStream::new();
        let mut draw_rx = self.draw_tx.subscribe();

        let event_stream = async_stream::stream! {
            loop {
                select! {
                    Some(Ok(event)) = crossterm_events.next() => {
                        match event {
                            Event::Key(key_event) => {
                                yield TuiEvent::Key(key_event);
                            }
                            Event::Resize(_, _) => {
                                yield TuiEvent::Draw;
                            }
                            Event::Paste(pasted) => {
                                yield TuiEvent::Paste(pasted);
                            }
                            _ => {}
                        }
                    }
                    result = draw_rx.recv() => {
                        match result {
                            Ok(_) => {
                                yield TuiEvent::Draw;
                            }
                            Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                                // Coalesce lagged events into a single draw
                                yield TuiEvent::Draw;
                            }
                            Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                                // Sender dropped; stop
                                break;
                            }
                        }
                    }
                }
            }
        };

        Box::pin(event_stream)
    }

    /// Draw the UI with the provided function.
    pub fn draw<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce(&mut ratatui::Frame),
    {
        self.terminal.draw(f)?;
        Ok(())
    }

    /// Clear the terminal.
    pub fn clear(&mut self) -> Result<()> {
        self.terminal.clear()?;
        Ok(())
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}

/// Handle for scheduling frame redraws.
#[derive(Clone, Debug)]
pub struct FrameRequester {
    frame_schedule_tx: tokio::sync::mpsc::UnboundedSender<Instant>,
}

impl FrameRequester {
    /// Schedule a frame to be drawn immediately.
    pub fn schedule_frame(&self) {
        let _ = self.frame_schedule_tx.send(Instant::now());
    }

    /// Schedule a frame to be drawn after a delay.
    pub fn schedule_frame_in(&self, dur: Duration) {
        let _ = self.frame_schedule_tx.send(Instant::now() + dur);
    }
}

/// Set a panic hook that restores the terminal before panicking.
fn set_panic_hook() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), DisableBracketedPaste);
        let _ = execute!(stdout(), LeaveAlternateScreen);
        original_hook(panic_info);
    }));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_requester_creation() {
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        let requester = FrameRequester {
            frame_schedule_tx: tx,
        };
        // Should not panic
        requester.schedule_frame();
    }
}
