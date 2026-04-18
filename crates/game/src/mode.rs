/// Top-level game mode — the single source of truth for what the game is
/// currently doing at the highest level.
///
/// Systems that behave differently across modes (input routing, physics,
/// audio, UI) should accept a `GameMode` parameter and `match` on it rather
/// than testing individual boolean flags.  Adding a new mode here is the only
/// change required to make it available to every such system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GameMode {
    /// Normal gameplay — all game systems run.
    #[default]
    Playing,
    /// Pause menu is open — game simulation is frozen, menu input is active.
    Paused,
}

impl GameMode {
    pub fn is_paused(self) -> bool {
        matches!(self, Self::Paused)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_playing() {
        assert_eq!(GameMode::default(), GameMode::Playing);
        assert!(!GameMode::Playing.is_paused());
    }

    #[test]
    fn paused_is_paused() {
        assert!(GameMode::Paused.is_paused());
    }
}
