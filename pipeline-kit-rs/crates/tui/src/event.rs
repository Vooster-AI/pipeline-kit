//! Event handling types for the TUI.
//!
//! This module defines the types used for event handling in the TUI,
//! including the EventStatus enum that widgets use to indicate whether
//! they consumed an event or not.

/// Status of an event after being handled by a widget.
///
/// Widgets return this enum from their `handle_key_event` methods to indicate
/// whether the event was consumed or should be passed to the next handler in
/// the chain of responsibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventStatus {
    /// The event was handled by the widget and should not be propagated further.
    Consumed,
    /// The event was not handled by the widget and should be passed to the next handler.
    NotConsumed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_status_equality() {
        assert_eq!(EventStatus::Consumed, EventStatus::Consumed);
        assert_eq!(EventStatus::NotConsumed, EventStatus::NotConsumed);
        assert_ne!(EventStatus::Consumed, EventStatus::NotConsumed);
    }

    #[test]
    fn test_event_status_copy() {
        let status1 = EventStatus::Consumed;
        let status2 = status1; // Should copy, not move
        assert_eq!(status1, status2);
    }
}
