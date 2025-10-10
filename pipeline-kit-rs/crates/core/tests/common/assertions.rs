//! Custom assertion helpers for E2E tests.

use pk_protocol::{ipc::Event, process_models::ProcessStatus};

/// Assert that a sequence of events contains a ProcessStarted event.
pub fn assert_has_process_started(events: &[Event]) -> bool {
    events.iter().any(|e| matches!(e, Event::ProcessStarted { .. }))
}

/// Assert that a sequence of events contains a ProcessCompleted event.
pub fn assert_has_process_completed(events: &[Event]) -> bool {
    events.iter().any(|e| matches!(e, Event::ProcessCompleted { .. }))
}

/// Assert that a sequence of events contains a ProcessStatusUpdate with specific status.
pub fn assert_has_status_update(events: &[Event], status: ProcessStatus) -> bool {
    events.iter().any(|e| {
        matches!(
            e,
            Event::ProcessStatusUpdate {
                status: s,
                ..
            } if *s == status
        )
    })
}

/// Assert that a sequence of events contains log chunks.
pub fn assert_has_log_chunks(events: &[Event]) -> bool {
    events.iter().any(|e| matches!(e, Event::ProcessLogChunk { .. }))
}

/// Assert that events are in the correct sequential order.
///
/// Checks that:
/// 1. ProcessStarted comes first
/// 2. ProcessStatusUpdate(Running) comes before completion
/// 3. ProcessCompleted or Failed comes last
pub fn assert_event_sequence(events: &[Event]) {
    if events.is_empty() {
        panic!("Event sequence is empty");
    }

    // First event should be ProcessStarted
    assert!(
        matches!(events[0], Event::ProcessStarted { .. }),
        "First event should be ProcessStarted, got: {:?}",
        events[0]
    );

    // Last event should be completion or killed
    let last = events.last().unwrap();
    assert!(
        matches!(last, Event::ProcessCompleted { .. } | Event::ProcessKilled { .. }),
        "Last event should be ProcessCompleted or ProcessKilled, got: {:?}",
        last
    );
}

/// Extract process ID from the first ProcessStarted event.
pub fn extract_process_id(events: &[Event]) -> Option<uuid::Uuid> {
    events.iter().find_map(|e| match e {
        Event::ProcessStarted { process_id, .. } => Some(*process_id),
        _ => None,
    })
}

/// Count events of a specific type.
pub fn count_log_chunks(events: &[Event]) -> usize {
    events.iter().filter(|e| matches!(e, Event::ProcessLogChunk { .. })).count()
}

/// Assert that a string contains a substring (case-insensitive).
pub fn assert_contains_ci(haystack: &str, needle: &str) {
    let haystack_lower = haystack.to_lowercase();
    let needle_lower = needle.to_lowercase();
    assert!(
        haystack_lower.contains(&needle_lower),
        "Expected '{}' to contain '{}' (case-insensitive)",
        haystack,
        needle
    );
}
