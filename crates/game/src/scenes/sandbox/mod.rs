mod world;

use std::cell::{Cell, RefCell};

use glam::Vec2;
use input::InputEvent;

use crate::{
    pause::PauseState,
    weapon::{WeaponFiringState, WeaponStats},
};
use world::World;

use super::{Scene, SceneTransition};

pub struct SandboxScene {
    world: World,
    pause: PauseState,
    stats_editor: RefCell<WeaponStats>,
    slow_motion: Cell<bool>,
    enemy_spawn_requests: Cell<u32>,
    player_respawn_requested: Cell<bool>,
}

impl SandboxScene {
    pub fn new() -> Self {
        Self {
            world: World::new(),
            pause: PauseState::new(),
            stats_editor: RefCell::new(WeaponStats::default()),
            slow_motion: Cell::new(false),
            enemy_spawn_requests: Cell::new(0),
            player_respawn_requested: Cell::new(false),
        }
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
        self.world.time_scale = if self.slow_motion.get() { 0.2 } else { 1.0 };
        if let Some(player) = self.world.players.first_mut() {
            player.weapon.stats = self.stats_editor.borrow().clone();
        }

        if self.player_respawn_requested.get() {
            self.player_respawn_requested.set(false);
            self.world.respawn_player(0);
        }

        let n = self.enemy_spawn_requests.get();
        if n > 0 {
            self.enemy_spawn_requests.set(0);
            let angle_step = std::f32::consts::TAU / n as f32;
            for i in 0..n {
                let angle = angle_step * i as f32;
                let pos = Vec2::new(angle.cos(), angle.sin()) * 4.0;
                self.world.spawn_enemy(pos);
            }
        }

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

        // ── Enemies panel ──────────────────────────────────────────────────────
        let alive = self.world.alive_enemy_count();
        let player_dead = self.world.player_health(0).map_or(false, |h| h <= 0.0);

        egui::Window::new("Enemies")
            .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-12.0, -12.0))
            .resizable(false)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.set_min_width(160.0);

                ui.label(format!("{alive} enemy alive"));

                ui.separator();

                if ui
                    .add_sized([ui.available_width(), 24.0], egui::Button::new("Spawn Dummy"))
                    .clicked()
                {
                    self.enemy_spawn_requests.set(self.enemy_spawn_requests.get() + 1);
                }

                let respawn_btn = egui::Button::new("Respawn P1");
                let respawn_btn = if player_dead {
                    respawn_btn.fill(egui::Color32::from_rgb(180, 60, 60))
                } else {
                    respawn_btn
                };
                if ui
                    .add_sized([ui.available_width(), 24.0], respawn_btn)
                    .clicked()
                {
                    self.player_respawn_requested.set(true);
                }
            });

        // Extract state display info as owned values so no long-lived borrow of self.world
        // is held across the egui closure.
        let state_display = self.world.players.first().map(|p| {
            let color = match &p.weapon.state {
                WeaponFiringState::Idle => egui::Color32::from_rgb(100, 220, 100),
                WeaponFiringState::Cooldown(_) => egui::Color32::from_rgb(220, 180, 60),
                WeaponFiringState::Burst { .. } => egui::Color32::from_rgb(220, 120, 60),
                WeaponFiringState::Reloading(_) => egui::Color32::from_rgb(160, 160, 220),
            };
            (p.weapon.state.label(), color, p.weapon.state.remaining_secs())
        });

        egui::Window::new("Weapon")
            .anchor(egui::Align2::LEFT_BOTTOM, egui::vec2(12.0, -12.0))
            .resizable(false)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.set_min_width(220.0);

                // State indicator
                if let Some((label, color, rem)) = state_display {
                    ui.horizontal(|ui| {
                        ui.label("State:");
                        ui.colored_label(color, label);
                        if rem > 0.0 {
                            ui.label(format!("({:.2}s)", rem));
                        }
                    });
                }

                ui.separator();

                // Slow-motion toggle
                let slow = self.slow_motion.get();
                let btn_text = if slow { "⏸  Normal speed" } else { "🐢  Slow motion  (0.2×)" };
                let btn = egui::Button::new(btn_text);
                let btn = if slow {
                    btn.fill(egui::Color32::from_rgb(40, 120, 60))
                } else {
                    btn
                };
                if ui.add_sized([ui.available_width(), 24.0], btn).clicked() {
                    self.slow_motion.set(!slow);
                }

                ui.separator();

                // Editable weapon stats
                let mut stats = self.stats_editor.borrow_mut();

                egui::Grid::new("weapon_stats_editor")
                    .num_columns(2)
                    .spacing([8.0, 3.0])
                    .show(ui, |ui| {
                        ui.label("Fire rate");
                        ui.add(
                            egui::DragValue::new(&mut stats.fire_rate)
                                .range(0.5..=60.0)
                                .suffix(" rps")
                                .speed(0.1),
                        );
                        ui.end_row();

                        ui.label("Projectiles");
                        ui.add(egui::Slider::new(&mut stats.projectiles_per_shot, 1u32..=16));
                        ui.end_row();

                        ui.label("Shot arc");
                        let mut arc_deg = stats.shot_arc.to_degrees();
                        if ui
                            .add(egui::Slider::new(&mut arc_deg, 0.0_f32..=180.0).suffix("°"))
                            .changed()
                        {
                            stats.shot_arc = arc_deg.to_radians();
                        }
                        ui.end_row();

                        ui.label("Jitter");
                        let mut jitter_deg = stats.jitter.to_degrees();
                        if ui
                            .add(egui::Slider::new(&mut jitter_deg, 0.0_f32..=45.0).suffix("°"))
                            .changed()
                        {
                            stats.jitter = jitter_deg.to_radians();
                        }
                        ui.end_row();

                        ui.label("Burst count");
                        ui.add(egui::Slider::new(&mut stats.burst_count, 1u32..=8));
                        ui.end_row();

                        ui.label("Burst delay");
                        let mut delay_ms = stats.burst_delay * 1000.0;
                        if ui
                            .add(
                                egui::DragValue::new(&mut delay_ms)
                                    .range(10.0_f32..=1000.0)
                                    .suffix(" ms")
                                    .speed(1.0),
                            )
                            .changed()
                        {
                            stats.burst_delay = delay_ms / 1000.0;
                        }
                        ui.end_row();

                        ui.label("Speed");
                        ui.add(
                            egui::DragValue::new(&mut stats.projectile_speed)
                                .range(1.0..=100.0)
                                .suffix(" u/s")
                                .speed(0.5),
                        );
                        ui.end_row();

                        ui.label("Size");
                        ui.add(
                            egui::DragValue::new(&mut stats.projectile_size)
                                .range(0.02..=0.5)
                                .speed(0.005),
                        );
                        ui.end_row();

                        ui.label("Lifetime");
                        ui.add(
                            egui::DragValue::new(&mut stats.projectile_lifetime)
                                .range(0.1..=10.0)
                                .suffix(" s")
                                .speed(0.05),
                        );
                        ui.end_row();

                        ui.label("Piercing");
                        ui.add(egui::Slider::new(&mut stats.piercing, 0u32..=10));
                        ui.end_row();

                        ui.label("Damage");
                        ui.add(
                            egui::DragValue::new(&mut stats.damage_total)
                                .range(0.0..=1000.0)
                                .speed(1.0),
                        );
                        ui.end_row();

                        ui.label("Recoil");
                        ui.add(
                            egui::DragValue::new(&mut stats.recoil_force)
                                .range(0.0..=20.0)
                                .speed(0.1),
                        );
                        ui.end_row();
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
