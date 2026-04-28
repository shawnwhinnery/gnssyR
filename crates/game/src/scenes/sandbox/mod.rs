mod world;

use input::InputEvent;

use crate::{pause::PauseState, weapon::WeaponFiringState};
use world::World;

use super::{Scene, SceneTransition};

pub struct SandboxScene {
    world: World,
    pause: PauseState,
}

impl SandboxScene {
    pub fn new() -> Self {
        Self { world: World::new(), pause: PauseState::new() }
    }
}

impl Default for SandboxScene {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene for SandboxScene {
    fn tick(&mut self, events: &[InputEvent]) -> Option<SceneTransition> {
        self.pause.tick(events);
        if !self.pause.is_paused() {
            self.world.tick(events);
        }
        None
    }

    fn draw(&self, driver: &mut dyn gfx::GraphicsDriver) {
        self.world.draw(driver);
    }

    fn draw_ui(&self, ctx: &egui::Context) {
        self.pause.draw_ui(ctx);

        if self.pause.is_paused() {
            return;
        }

        if let Some(player) = self.world.players.first() {
            let stats = &player.weapon.stats;
            let state = &player.weapon.state;

            egui::Window::new("Weapon")
                .anchor(egui::Align2::LEFT_BOTTOM, egui::vec2(12.0, -12.0))
                .resizable(false)
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.set_min_width(160.0);

                    let state_color = match state {
                        WeaponFiringState::Idle => egui::Color32::from_rgb(100, 220, 100),
                        WeaponFiringState::Cooldown(_) => egui::Color32::from_rgb(220, 180, 60),
                        WeaponFiringState::Burst { .. } => egui::Color32::from_rgb(220, 120, 60),
                        WeaponFiringState::Reloading(_) => egui::Color32::from_rgb(160, 160, 220),
                    };

                    ui.horizontal(|ui| {
                        ui.label("State:");
                        ui.colored_label(state_color, state.label());
                        let rem = state.remaining_secs();
                        if rem > 0.0 {
                            ui.label(format!("({:.2}s)", rem));
                        }
                    });

                    ui.separator();

                    egui::Grid::new("weapon_stats")
                        .num_columns(2)
                        .spacing([8.0, 2.0])
                        .show(ui, |ui| {
                            ui.label("Fire rate");
                            ui.label(format!("{:.1} rps", stats.fire_rate));
                            ui.end_row();

                            ui.label("Projectiles");
                            ui.label(format!("{}", stats.projectiles_per_shot));
                            ui.end_row();

                            if stats.burst_count > 1 {
                                ui.label("Burst");
                                ui.label(format!(
                                    "{}× @ {:.0}ms",
                                    stats.burst_count,
                                    stats.burst_delay * 1000.0
                                ));
                                ui.end_row();
                            }

                            if stats.shot_arc > 0.0 {
                                ui.label("Arc");
                                ui.label(format!(
                                    "{:.0}°",
                                    stats.shot_arc.to_degrees()
                                ));
                                ui.end_row();
                            }

                            ui.label("Speed");
                            ui.label(format!("{:.0}", stats.projectile_speed));
                            ui.end_row();

                            ui.label("Lifetime");
                            ui.label(format!("{:.1}s", stats.projectile_lifetime));
                            ui.end_row();

                            ui.label("Damage");
                            ui.label(format!("{:.0}", stats.damage_total));
                            ui.end_row();

                            if stats.piercing > 0 {
                                ui.label("Piercing");
                                ui.label(format!("{}", stats.piercing));
                                ui.end_row();
                            }
                        });

                    ui.separator();
                    ui.label(
                        egui::RichText::new("LMB / Space to fire")
                            .small()
                            .color(egui::Color32::GRAY),
                    );
                });
        }
    }
}
