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
