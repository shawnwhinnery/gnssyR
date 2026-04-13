use crate::event::InputEvent;

/// Swappable input source.
///
/// Implementations: [`crate::gilrs_backend::GilrsBackend`] (real hardware),
/// [`crate::simulated::SimulatedBackend`] (inject events in tests).
pub trait InputBackend {
    /// Drain all pending events since the last call.
    fn poll(&mut self) -> Vec<InputEvent>;
}
