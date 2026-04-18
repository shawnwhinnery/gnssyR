use crate::{backend::InputBackend, event::InputEvent};

/// Test-only backend that lets callers inject [`InputEvent`]s directly.
///
/// Use this in place of [`crate::gilrs_backend::GilrsBackend`] in headless
/// tests and the automated test pipeline — no hardware or OS input required.
#[derive(Default)]
pub struct SimulatedBackend {
    queue: Vec<InputEvent>,
}

impl SimulatedBackend {
    pub fn new() -> Self {
        Self::default()
    }

    /// Enqueue an event to be returned on the next [`poll`](Self::poll) call.
    pub fn push(&mut self, event: InputEvent) {
        self.queue.push(event);
    }

    /// Enqueue multiple events at once.
    pub fn push_all(&mut self, events: impl IntoIterator<Item = InputEvent>) {
        self.queue.extend(events);
    }
}

impl InputBackend for SimulatedBackend {
    fn poll(&mut self) -> Vec<InputEvent> {
        std::mem::take(&mut self.queue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::InputEvent;

    // M1 — push CursorMoved then poll returns it.
    #[test]
    fn cursor_moved_push_then_poll() {
        let mut b = SimulatedBackend::new();
        b.push(InputEvent::CursorMoved { x: 0.5, y: -0.3 });
        let events = b.poll();
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], InputEvent::CursorMoved { x, y } if x == 0.5 && y == -0.3));
    }

    // M2 — two CursorMoved events returned in insertion order.
    #[test]
    fn cursor_moved_two_in_order() {
        let mut b = SimulatedBackend::new();
        b.push(InputEvent::CursorMoved { x: -1.0, y: -1.0 });
        b.push(InputEvent::CursorMoved { x: 1.0, y: 1.0 });
        let events = b.poll();
        assert_eq!(events.len(), 2);
        assert!(matches!(events[0], InputEvent::CursorMoved { x, y } if x == -1.0 && y == -1.0));
        assert!(matches!(events[1], InputEvent::CursorMoved { x, y } if x ==  1.0 && y ==  1.0));
    }

    // M3 — corner values (1.0, 1.0) are returned as-is; no clamping at the input layer.
    #[test]
    fn cursor_moved_corner_passthrough() {
        let mut b = SimulatedBackend::new();
        b.push(InputEvent::CursorMoved { x: 1.0, y: 1.0 });
        let events = b.poll();
        assert!(matches!(events[0], InputEvent::CursorMoved { x, y } if x == 1.0 && y == 1.0));
    }

    // M4 — MouseMove and CursorMoved mixed; both returned in insertion order.
    #[test]
    fn mouse_move_and_cursor_moved_mixed() {
        let mut b = SimulatedBackend::new();
        b.push(InputEvent::MouseMove { dx: 3.0, dy: 4.0 });
        b.push(InputEvent::CursorMoved { x: 0.2, y: 0.7 });
        let events = b.poll();
        assert_eq!(events.len(), 2);
        assert!(matches!(events[0], InputEvent::MouseMove { dx, dy } if dx == 3.0 && dy == 4.0));
        assert!(matches!(events[1], InputEvent::CursorMoved { x, y } if x == 0.2 && y == 0.7));
    }

    // M5 — poll with no prior push returns empty Vec.
    #[test]
    fn cursor_moved_empty_poll() {
        let mut b = SimulatedBackend::new();
        assert!(b.poll().is_empty());
    }
}
