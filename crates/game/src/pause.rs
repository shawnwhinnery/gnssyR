use crate::mode::GameMode;
use input::event::{Button, InputEvent, KeyCode};
use std::cell::Cell;

/// Scene-agnostic pause controller.
///
/// Owns the current [`GameMode`] and is responsible for the Escape-key
/// transition between `Playing` and `Paused`.
///
/// Usage in a game loop:
/// 1. Call [`PauseState::tick`] with the raw event slice first — it updates
///    the mode before any scene logic runs.
/// 2. Read [`PauseState::mode`] to route events and simulation correctly.
/// 3. Call [`PauseState::draw_ui`] with the egui context to render the overlay.
///
/// `mode` is stored in a [`Cell`] so [`draw_ui`](Self::draw_ui) can toggle it
/// via the Resume button without requiring `&mut self`.
pub struct PauseState {
    mode: Cell<GameMode>,
}

impl PauseState {
    pub fn new() -> Self {
        Self {
            mode: Cell::new(GameMode::default()),
        }
    }

    /// The current top-level game mode.
    pub fn mode(&self) -> GameMode {
        self.mode.get()
    }

    /// Convenience wrapper — prefer matching on [`mode`](Self::mode) when
    /// there are multiple modes to handle.
    pub fn is_paused(&self) -> bool {
        self.mode.get().is_paused()
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
                self.mode.set(match self.mode.get() {
                    GameMode::Playing => GameMode::Paused,
                    GameMode::Paused => GameMode::Playing,
                });
            }
        }
    }

    /// Draw the egui pause overlay when in `Paused` mode.
    ///
    /// Shows a centered modal with a Resume button. The Resume button writes
    /// back through [`Cell`] so this works from a shared `&self` reference
    /// (required by [`Scene::draw_ui`]).
    pub fn draw_ui(&self, ctx: &egui::Context) {
        if !self.is_paused() {
            return;
        }

        let frame = egui::Frame::window(&ctx.style())
            .fill(egui::Color32::from_rgba_premultiplied(10, 10, 10, 230))
            .inner_margin(egui::Margin::symmetric(40.0, 24.0));

        egui::Window::new("##pause_overlay")
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .resizable(false)
            .collapsible(false)
            .title_bar(false)
            .frame(frame)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("PAUSED");
                    ui.add_space(16.0);
                    if ui
                        .add_sized([120.0, 36.0], egui::Button::new("Resume"))
                        .clicked()
                    {
                        self.mode.set(GameMode::Playing);
                    }
                });
            });
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
        p.tick(&[escape_up()]); // → still Paused (key-up ignored)
        assert_eq!(p.mode(), GameMode::Paused);
    }
}
