use crate::{
    loot::WeaponStatRarities,
    mode::GameMode,
    weapon::{ProjectileBehavior, WeaponStats},
};
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
    /// `weapon` is `(name, stats, behavior)` for the player's primary weapon.
    /// Pass `None` when no player is active.
    pub fn draw_ui(
        &self,
        ctx: &egui::Context,
        weapon: Option<(&str, &WeaponStats, ProjectileBehavior)>,
    ) {
        if !self.is_paused() {
            return;
        }

        let frame = egui::Frame::window(&ctx.style())
            .fill(egui::Color32::from_rgba_premultiplied(10, 10, 10, 230))
            .inner_margin(egui::Margin::symmetric(32.0, 24.0));

        egui::Window::new("##pause_overlay")
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .resizable(false)
            .collapsible(false)
            .title_bar(false)
            .min_width(520.0)
            .frame(frame)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(
                        egui::RichText::new("CHARACTER")
                            .heading()
                            .color(egui::Color32::WHITE),
                    );
                });
                ui.add_space(8.0);
                ui.separator();
                ui.add_space(10.0);

                ui.columns(2, |cols| {
                    draw_weapon_column(&mut cols[0], weapon);
                    // Right column reserved for future stats.
                    cols[1].label("");
                });

                ui.add_space(12.0);
                ui.separator();
                ui.add_space(8.0);

                ui.vertical_centered(|ui| {
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

// ---------------------------------------------------------------------------
// Weapon stat column
// ---------------------------------------------------------------------------

fn draw_weapon_column(
    ui: &mut egui::Ui,
    weapon: Option<(&str, &WeaponStats, ProjectileBehavior)>,
) {
    ui.label(
        egui::RichText::new("PRIMARY WEAPON")
            .strong()
            .color(egui::Color32::from_rgb(180, 180, 180)),
    );
    ui.add_space(4.0);

    let Some((name, stats, behavior)) = weapon else {
        ui.label(egui::RichText::new("—").color(egui::Color32::DARK_GRAY));
        return;
    };

    let r = WeaponStatRarities::from_stats(stats);

    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(name)
                .italics()
                .color(rarity_color(r.overall_score())),
        );
        ui.label(
            egui::RichText::new(format!("({})", behavior.label()))
                .color(egui::Color32::GRAY),
        );
    });
    ui.add_space(6.0);

    let bounces_text = if stats.physics_max_bounces == 0 {
        "∞".to_string()
    } else {
        stats.physics_max_bounces.to_string()
    };

    egui::ScrollArea::vertical()
        .max_height(340.0)
        .show(ui, |ui| {
            egui::Grid::new("pause_weapon_table")
                .num_columns(4)
                .spacing([8.0, 3.0])
                .min_col_width(0.0)
                .striped(true)
                .show(ui, |ui| {
                    // Header
                    for h in ["Stat", "Base", "Mods", "Total"] {
                        ui.label(egui::RichText::new(h).weak());
                    }
                    ui.end_row();

                    // ── Firing ──────────────────────────────────────────────
                    section(ui, "Firing");
                    row(ui, "Fire Rate",   &format!("{:.1} rps", stats.fire_rate),                 r.fire_rate);
                    row(ui, "Burst Count", &stats.burst_count.to_string(),                          r.burst_count);
                    row(ui, "Burst Delay", &format!("{:.0} ms", stats.burst_delay * 1000.0),        r.burst_delay);

                    // ── Spread ──────────────────────────────────────────────
                    section(ui, "Spread");
                    row(ui, "Pellets",     &stats.projectiles_per_shot.to_string(),                r.projectiles_per_shot);
                    row(ui, "Shot Arc",    &format!("{:.1}°", stats.shot_arc.to_degrees()),         0.0);
                    row(ui, "Jitter",      &format!("{:.1}°", stats.jitter.to_degrees()),           r.jitter);
                    row(ui, "Kickback",    &format!("{:.2}°", stats.kickback.to_degrees()),         0.0);
                    row(ui, "Sway (τ)",    &format!("{:.2} s", stats.sway),                        r.sway);

                    // ── Projectile ──────────────────────────────────────────
                    section(ui, "Projectile");
                    row(ui, "Speed",       &format!("{:.0} u/s", stats.projectile_speed),           r.projectile_speed);
                    row(ui, "Size",        &format!("{:.3}", stats.projectile_size),                r.projectile_size);
                    row(ui, "Lifetime",    &format!("{:.1} s", stats.projectile_lifetime),          0.0);
                    row(ui, "Piercing",    &stats.piercing.to_string(),                             0.0);

                    // ── Impact ──────────────────────────────────────────────
                    section(ui, "Impact");
                    row(ui, "Damage",      &format!("{:.1}", stats.damage_total),                   r.damage_total);
                    row(ui, "Recoil",      &format!("{:.2}", stats.recoil_force),                   r.recoil_force);

                    // ── Oscillating ─────────────────────────────────────────
                    section(ui, "Oscillating");
                    row(ui, "Frequency",   &format!("{:.1} Hz", stats.oscillation_frequency),       0.0);
                    row(ui, "Magnitude",   &format!("{:.3}", stats.oscillation_magnitude),          0.0);

                    // ── Physics ─────────────────────────────────────────────
                    section(ui, "Physics");
                    row(ui, "Max Bounces", &bounces_text,                                           0.0);
                    row(ui, "Friction",    &format!("{:.3}", stats.physics_friction),               0.0);
                    row(ui, "Min Speed",   &format!("{:.2} u/s", stats.physics_min_speed),          0.0);

                    // ── Rocket ───────────────────────────────────────────────
                    section(ui, "Rocket");
                    row(ui, "Acceleration",&format!("{:.0} u/s²", stats.rocket_acceleration),       0.0);
                    row(ui, "Kinetic Scale",&format!("{:.3}", stats.kinetic_damage_scale),           0.0);

                    // ── Seeking ──────────────────────────────────────────────
                    section(ui, "Seeking");
                    row(ui, "Turn Radius", &format!("{:.1} u", stats.seeking_turn_radius),          0.0);
                    row(ui, "Acquire Angle",&format!("{:.0}°", stats.seeking_acquire_half_angle.to_degrees()), 0.0);
                });
        });
}

// ---------------------------------------------------------------------------
// Table helpers
// ---------------------------------------------------------------------------

pub(crate) fn rarity_color(fraction: f32) -> egui::Color32 {
    if fraction >= 0.80 {
        egui::Color32::from_rgb(220, 50, 50)  // mythic  – red
    } else if fraction >= 0.60 {
        egui::Color32::from_rgb(255, 175, 0)  // epic    – gold
    } else if fraction >= 0.40 {
        egui::Color32::from_rgb(160, 50, 235) // rare    – purple
    } else if fraction >= 0.20 {
        egui::Color32::from_rgb(60, 115, 255) // uncommon – blue
    } else {
        egui::Color32::from_rgb(130, 130, 130) // common  – grey
    }
}

/// One data row: stat label | base (rarity-coloured) | mods (—) | total.
fn row(ui: &mut egui::Ui, label: &str, value: &str, rarity: f32) {
    ui.label(egui::RichText::new(label).color(egui::Color32::GRAY));
    ui.label(egui::RichText::new(value).color(rarity_color(rarity)));
    ui.label(egui::RichText::new("—").color(egui::Color32::DARK_GRAY));
    ui.label(value); // total = base + 0 mods
    ui.end_row();
}

/// Section header row spanning all 4 columns.
fn section(ui: &mut egui::Ui, title: &str) {
    ui.label(
        egui::RichText::new(title)
            .strong()
            .color(egui::Color32::from_rgb(155, 155, 155)),
    );
    for _ in 0..3 {
        ui.label("");
    }
    ui.end_row();
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

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
        let mut p = PauseState::new();
        p.tick(&[escape_down()]); // → Paused
        p.tick(&[escape_up()]); // → still Paused (key-up ignored)
        assert_eq!(p.mode(), GameMode::Paused);
    }
}
