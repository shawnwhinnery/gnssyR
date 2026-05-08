use std::cell::Cell;

use gfx::Color;
use glam::Vec2;
use input::InputEvent;
use physics::{Body, BodyHandle, Collider};

use crate::{pause::PauseState, world::World};

use super::{Scene, SceneTransition};

// ---------------------------------------------------------------------------
// Layout constants (world units; camera HALF_VIEW = 5.6)
// ---------------------------------------------------------------------------

// Two rooms side-by-side, each ≈12×12 world units.
//   Room 1: x ∈ [-14, 0]   Room 2: x ∈ [0, 14]   y ∈ [-6, 6]
// The divider wall has a 3-unit door gap centred at y = 0.

const WALL_COLOR: u32 = 0x3A4A5AFF;
const DIVIDER_COLOR: u32 = 0x2A3848FF;
const DOOR_CLOSED_COLOR: u32 = 0xCC8833FF;
const PILLAR_COLOR: u32 = 0x55667788;

const DOOR_HALF_H: f32 = 1.5; // half the door gap height
const DOOR_HALF_W: f32 = 0.25; // half the divider wall thickness

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn rect(hw: f32, hh: f32) -> Vec<Vec2> {
    vec![
        Vec2::new(-hw, -hh),
        Vec2::new(hw, -hh),
        Vec2::new(hw, hh),
        Vec2::new(-hw, hh),
    ]
}

fn static_wall(position: Vec2, collider: Collider) -> Body {
    let (collision_layers, collision_mask) = crate::physics_layers::wall_collision();
    Body {
        position,
        velocity: Vec2::ZERO,
        mass: f32::INFINITY,
        restitution: 0.3,
        collision_layers,
        collision_mask,
        collider,
    }
}

// ---------------------------------------------------------------------------
// Phase
// ---------------------------------------------------------------------------

#[derive(Copy, Clone, PartialEq)]
enum Phase {
    Room1,
    DoorOpen,
    Room2,
    Win,
    GameOver,
}

// ---------------------------------------------------------------------------
// Level1Scene
// ---------------------------------------------------------------------------

pub struct Level1Scene {
    world: World,
    pause: PauseState,
    phase: Phase,
    door_body: Option<BodyHandle>,
    room1_handles: Vec<BodyHandle>,
    room2_handles: Vec<BodyHandle>,
    return_to_menu: Cell<bool>,
    restart: Cell<bool>,
}

impl Level1Scene {
    pub fn new() -> Self {
        let mut world = World::new();

        // Place P1 at the centre-left of room 1.
        world.physics.body_mut(world.players[0].actor.body).position = Vec2::new(-9.0, 0.0);

        // ── Outer boundary ───────────────────────────────────────────────────
        // Top wall  (spans both rooms)
        world.add_wall(
            static_wall(
                Vec2::new(0.0, 6.25),
                Collider::Convex {
                    vertices: rect(14.25, 0.25),
                },
            ),
            'W',
            Color::hex(WALL_COLOR),
        );
        // Bottom wall
        world.add_wall(
            static_wall(
                Vec2::new(0.0, -6.25),
                Collider::Convex {
                    vertices: rect(14.25, 0.25),
                },
            ),
            'W',
            Color::hex(WALL_COLOR),
        );
        // Left wall
        world.add_wall(
            static_wall(
                Vec2::new(-14.25, 0.0),
                Collider::Convex {
                    vertices: rect(0.25, 6.25),
                },
            ),
            'W',
            Color::hex(WALL_COLOR),
        );
        // Right wall
        world.add_wall(
            static_wall(
                Vec2::new(14.25, 0.0),
                Collider::Convex {
                    vertices: rect(0.25, 6.25),
                },
            ),
            'W',
            Color::hex(WALL_COLOR),
        );

        // ── Divider (y: 1.5..6 and -6..-1.5, gap at centre) ─────────────────
        let div_top_y = (6.0 + DOOR_HALF_H) / 2.0; // centre of upper segment
        let div_top_hh = (6.0 - DOOR_HALF_H) / 2.0;
        world.add_wall(
            static_wall(
                Vec2::new(0.0, div_top_y),
                Collider::Convex {
                    vertices: rect(DOOR_HALF_W, div_top_hh),
                },
            ),
            'W',
            Color::hex(DIVIDER_COLOR),
        );
        world.add_wall(
            static_wall(
                Vec2::new(0.0, -div_top_y),
                Collider::Convex {
                    vertices: rect(DOOR_HALF_W, div_top_hh),
                },
            ),
            'W',
            Color::hex(DIVIDER_COLOR),
        );

        // ── Door (fills the gap, closed at start) ────────────────────────────
        let door_handle = world.add_wall(
            static_wall(
                Vec2::new(0.0, 0.0),
                Collider::Convex {
                    vertices: rect(DOOR_HALF_W, DOOR_HALF_H),
                },
            ),
            'D',
            Color::hex(DOOR_CLOSED_COLOR),
        );

        // ── Cover pillars (one per room) ─────────────────────────────────────
        world.add_wall(
            static_wall(Vec2::new(-7.0, 2.0), Collider::Circle { radius: 0.7 }),
            'P',
            Color::hex(PILLAR_COLOR),
        );
        world.add_wall(
            static_wall(Vec2::new(7.0, -2.0), Collider::Circle { radius: 0.7 }),
            'P',
            Color::hex(PILLAR_COLOR),
        );

        // ── Room 1 enemies ───────────────────────────────────────────────────
        let room1_positions = [Vec2::new(-5.0, 3.5), Vec2::new(-11.0, -3.5)];
        let mut room1_handles = Vec::new();
        for pos in &room1_positions {
            world.spawn_enemy(*pos);
            room1_handles.push(world.enemies.last().unwrap().body());
        }

        Self {
            world,
            pause: PauseState::new(),
            phase: Phase::Room1,
            door_body: Some(door_handle),
            room1_handles,
            room2_handles: Vec::new(),
            return_to_menu: Cell::new(false),
            restart: Cell::new(false),
        }
    }

    // ── Phase helpers ────────────────────────────────────────────────────────

    fn room1_cleared(&self) -> bool {
        !self.room1_handles.is_empty()
            && self
                .room1_handles
                .iter()
                .all(|&h| !self.world.enemies.iter().any(|e| e.body() == h))
    }

    fn room2_cleared(&self) -> bool {
        !self.room2_handles.is_empty()
            && self
                .room2_handles
                .iter()
                .all(|&h| !self.world.enemies.iter().any(|e| e.body() == h))
    }

    fn room1_alive_count(&self) -> usize {
        self.room1_handles
            .iter()
            .filter(|&&h| self.world.enemies.iter().any(|e| e.body() == h))
            .count()
    }

    fn room2_alive_count(&self) -> usize {
        self.room2_handles
            .iter()
            .filter(|&&h| self.world.enemies.iter().any(|e| e.body() == h))
            .count()
    }

    fn any_player_in_room2(&self) -> bool {
        self.world
            .players
            .iter()
            .any(|p| self.world.physics.body(p.actor.body).position.x > 0.5)
    }

    // ── Door control ─────────────────────────────────────────────────────────

    fn open_door(&mut self) {
        if let Some(h) = self.door_body.take() {
            self.world.remove_wall(h);
        }
    }

    fn close_door(&mut self) {
        if self.door_body.is_none() {
            let h = self.world.add_wall(
                static_wall(
                    Vec2::new(0.0, 0.0),
                    Collider::Convex {
                        vertices: rect(DOOR_HALF_W, DOOR_HALF_H),
                    },
                ),
                'D',
                Color::hex(DOOR_CLOSED_COLOR),
            );
            self.door_body = Some(h);
        }
    }

    // ── Spawn room 2 enemies and close door behind the player ────────────────
    fn enter_room2(&mut self) {
        self.close_door();
        let positions = [Vec2::new(5.0, -3.5), Vec2::new(11.0, 3.5)];
        for pos in &positions {
            self.world.spawn_enemy(*pos);
            self.room2_handles
                .push(self.world.enemies.last().unwrap().body());
        }
    }
}

impl Default for Level1Scene {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene for Level1Scene {
    fn tick(&mut self, events: &[InputEvent]) -> Option<SceneTransition> {
        self.pause.tick(events);

        // Deferred UI button actions (set from &self in draw_ui).
        if self.return_to_menu.get() {
            return Some(SceneTransition::Replace(Box::new(
                crate::scenes::main_menu::MainMenuScene::new(),
            )));
        }
        if self.restart.get() {
            return Some(SceneTransition::Replace(Box::new(Level1Scene::new())));
        }

        if !self.pause.is_paused() {
            self.world.tick(events);
        }

        // Game-over check (only during active fight phases).
        if matches!(self.phase, Phase::Room1 | Phase::Room2) {
            if self.world.players.iter().all(|p| p.health <= 0.0) {
                self.phase = Phase::GameOver;
                return None;
            }
        }

        // State-machine transitions.
        match self.phase {
            Phase::Room1 => {
                if self.room1_cleared() {
                    self.phase = Phase::DoorOpen;
                    self.open_door();
                }
            }
            Phase::DoorOpen => {
                if self.any_player_in_room2() {
                    self.phase = Phase::Room2;
                    self.enter_room2();
                }
            }
            Phase::Room2 => {
                if self.room2_cleared() {
                    self.phase = Phase::Win;
                }
            }
            Phase::Win | Phase::GameOver => {}
        }

        None
    }

    fn draw(&self, driver: &mut dyn gfx::GraphicsDriver) {
        self.world.draw(driver);
    }

    fn draw_ui(&self, ctx: &egui::Context) {
        let weapon_display = self.world.players.first().map(|p| {
            (
                p.weapon_name.as_deref().unwrap_or("Default Loadout"),
                &p.weapon.stats,
                p.weapon.projectile_behavior,
            )
        });
        self.pause.draw_ui(ctx, weapon_display);
        if self.pause.is_paused() {
            return;
        }

        // ── Phase banner (top-centre) ─────────────────────────────────────────
        let (banner, banner_color) = match self.phase {
            Phase::Room1 => {
                let n = self.room1_alive_count();
                (
                    format!("CLEAR THE ROOM  —  {n} enemies"),
                    egui::Color32::WHITE,
                )
            }
            Phase::DoorOpen => (
                "ADVANCE!".to_string(),
                egui::Color32::from_rgb(100, 220, 100),
            ),
            Phase::Room2 => {
                let n = self.room2_alive_count();
                (format!("FINAL ROOM  —  {n} enemies"), egui::Color32::WHITE)
            }
            Phase::Win | Phase::GameOver => (String::new(), egui::Color32::WHITE),
        };
        if !banner.is_empty() {
            egui::Area::new(egui::Id::new("level1_banner"))
                .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 12.0))
                .show(ctx, |ui| {
                    ui.label(
                        egui::RichText::new(&banner)
                            .size(22.0)
                            .color(banner_color)
                            .strong(),
                    );
                });
        }

        // ── P1 health bar (top-left) ─────────────────────────────────────────
        let hp = self.world.player_health(0).unwrap_or(0.0);
        egui::Area::new(egui::Id::new("level1_hp"))
            .anchor(egui::Align2::LEFT_TOP, egui::vec2(12.0, 12.0))
            .show(ctx, |ui| {
                ui.label(egui::RichText::new("P1").color(egui::Color32::from_rgb(41, 121, 255)));
                ui.add(
                    egui::ProgressBar::new(hp / 100.0)
                        .desired_width(120.0)
                        .fill(egui::Color32::from_rgb(60, 180, 80)),
                );
            });

        // ── Win overlay ───────────────────────────────────────────────────────
        if matches!(self.phase, Phase::Win) {
            let frame = egui::Frame::window(&ctx.style())
                .fill(egui::Color32::from_rgba_premultiplied(10, 10, 20, 220))
                .inner_margin(egui::Margin::symmetric(48.0, 32.0));
            egui::Window::new("##win")
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .resizable(false)
                .collapsible(false)
                .title_bar(false)
                .frame(frame)
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new("YOU WIN!")
                                .size(48.0)
                                .color(egui::Color32::GOLD)
                                .strong(),
                        );
                        ui.add_space(16.0);
                        if ui
                            .add_sized([160.0, 36.0], egui::Button::new("Return to Menu"))
                            .clicked()
                        {
                            self.return_to_menu.set(true);
                        }
                    });
                });
        }

        // ── Game-over overlay ─────────────────────────────────────────────────
        if matches!(self.phase, Phase::GameOver) {
            let frame = egui::Frame::window(&ctx.style())
                .fill(egui::Color32::from_rgba_premultiplied(30, 5, 5, 220))
                .inner_margin(egui::Margin::symmetric(48.0, 32.0));
            egui::Window::new("##gameover")
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .resizable(false)
                .collapsible(false)
                .title_bar(false)
                .frame(frame)
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new("GAME OVER")
                                .size(48.0)
                                .color(egui::Color32::from_rgb(220, 60, 60))
                                .strong(),
                        );
                        ui.add_space(16.0);
                        if ui
                            .add_sized([160.0, 36.0], egui::Button::new("Try Again"))
                            .clicked()
                        {
                            self.restart.set(true);
                        }
                        ui.add_space(8.0);
                        if ui
                            .add_sized([160.0, 36.0], egui::Button::new("Return to Menu"))
                            .clicked()
                        {
                            self.return_to_menu.set(true);
                        }
                    });
                });
        }
    }
}
