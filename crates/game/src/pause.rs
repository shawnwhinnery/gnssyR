use crate::mode::GameMode;
use input::event::{Button, InputEvent, KeyCode};

/// Scene-agnostic pause controller.
///
/// Owns the current [`GameMode`] and is responsible for the Escape-key
/// transition between `Playing` and `Paused`.
///
/// Usage in a game loop:
/// 1. Call [`PauseState::tick`] with the raw event slice first — it updates
///    the mode before any scene logic runs.
/// 2. Read [`PauseState::mode`] to route events and simulation correctly.
/// 3. Call [`PauseState::draw`] after your scene renders to layer the overlay.
pub struct PauseState {
    mode: GameMode,
}

impl PauseState {
    pub fn new() -> Self {
        Self { mode: GameMode::default() }
    }

    /// The current top-level game mode.
    pub fn mode(&self) -> GameMode {
        self.mode
    }

    /// Convenience wrapper — prefer matching on [`mode`](Self::mode) when
    /// there are multiple modes to handle.
    pub fn is_paused(&self) -> bool {
        self.mode.is_paused()
    }

    /// Scan `events` for Escape key-down and toggle between `Playing` and
    /// `Paused`.
    ///
    /// Only `pressed: true` is acted upon — the key-up event is ignored.
    /// This prevents a single physical key press from toggling the menu twice
    /// in the same logical frame.
    pub fn tick(&mut self, events: &[InputEvent]) {
        for event in events {
            if let InputEvent::Button {
                button: Button::Key(KeyCode::Escape),
                pressed: true,
                ..
            } = event
            {
                self.mode = match self.mode {
                    GameMode::Playing => GameMode::Paused,
                    GameMode::Paused => GameMode::Playing,
                };
            }
        }
    }

    /// Draw the pause overlay when in `Paused` mode. Must be called after the
    /// scene renders so the modal sits on top of the frozen world.
    pub fn draw(&self, driver: &mut dyn gfx::GraphicsDriver) {
        if self.mode.is_paused() {
            crate::hud::draw_pause_overlay(driver);
        }
    }
}

impl Default for PauseState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use input::player::PlayerId;

    fn escape_down() -> InputEvent {
        InputEvent::Button {
            player: PlayerId::P1,
            button: Button::Key(KeyCode::Escape),
            pressed: true,
        }
    }

    fn escape_up() -> InputEvent {
        InputEvent::Button {
            player: PlayerId::P1,
            button: Button::Key(KeyCode::Escape),
            pressed: false,
        }
    }

    #[test]
    fn starts_playing() {
        assert_eq!(PauseState::new().mode(), GameMode::Playing);
    }

    #[test]
    fn escape_down_transitions_to_paused() {
        let mut p = PauseState::new();
        p.tick(&[escape_down()]);
        assert_eq!(p.mode(), GameMode::Paused);
    }

    #[test]
    fn escape_down_twice_returns_to_playing() {
        let mut p = PauseState::new();
        p.tick(&[escape_down()]);
        p.tick(&[escape_down()]);
        assert_eq!(p.mode(), GameMode::Playing);
    }

    #[test]
    fn key_up_does_not_toggle() {
        let mut p = PauseState::new();
        p.tick(&[escape_up()]);
        assert_eq!(p.mode(), GameMode::Playing);
    }

    #[test]
    fn key_down_then_up_does_not_double_trigger() {
        // Simulates the OS delivering press + release in separate ticks.
        let mut p = PauseState::new();
        p.tick(&[escape_down()]); // → Paused
        p.tick(&[escape_up()]);   // → still Paused (key-up ignored)
        assert_eq!(p.mode(), GameMode::Paused);
    }
}
