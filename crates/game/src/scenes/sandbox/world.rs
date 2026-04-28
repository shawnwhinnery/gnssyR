use glam::{Mat3, Vec2};
use input::InputEvent;
use physics::{Body, BodyHandle, Collider, PhysicsWorld};

use crate::{
    camera::Camera,
    hud,
    input::InputState,
    player::{draw_players, Player, PLAYER_SPEED},
    weapon::Projectile,
};
use gfx::{
    shape::{circle, polygon},
    style::{Fill, LineCap, LineJoin, Stroke, Style},
    tessellate, Color,
};

pub(super) const GROUND_COLOR: [f32; 4] = [0.13, 0.14, 0.12, 1.0];

/// Projectiles spawn this many world units in front of the player's edge.
const PLAYER_RADIUS_SPAWN_OFFSET: f32 = crate::player::PLAYER_RADIUS + 0.05;

// ---------------------------------------------------------------------------
// Walls
// ---------------------------------------------------------------------------

pub struct Wall {
    pub body: BodyHandle,
    /// Single-char label shown in the collision HUD (C=circle, R=rect, T=triangle, O=octagon).
    pub label: char,
    pub fill_color: Color,
}

fn make_walls(physics: &mut PhysicsWorld) -> Vec<Wall> {
    let mut walls = Vec::new();

    // Circle — right side.
    let h = physics.add_body(Body {
        position: Vec2::new(3.0, 0.0),
        velocity: Vec2::ZERO,
        mass: f32::INFINITY,
        restitution: 0.3,
        collider: Collider::Circle { radius: 0.65 },
    });
    walls.push(Wall { body: h, label: 'C', fill_color: Color::hex(0x7C4DFF99) });

    // Rectangle — left side.
    let h = physics.add_body(Body {
        position: Vec2::new(-3.0, 0.0),
        velocity: Vec2::ZERO,
        mass: f32::INFINITY,
        restitution: 0.3,
        collider: Collider::Convex {
            vertices: vec![
                Vec2::new(-0.8, -0.5),
                Vec2::new(0.8, -0.5),
                Vec2::new(0.8, 0.5),
                Vec2::new(-0.8, 0.5),
            ],
        },
    });
    walls.push(Wall { body: h, label: 'R', fill_color: Color::hex(0xFF6D0099) });

    // Triangle — top side (equilateral, CCW, circumradius 0.75).
    let r = 0.75_f32;
    let h = physics.add_body(Body {
        position: Vec2::new(0.0, 3.0),
        velocity: Vec2::ZERO,
        mass: f32::INFINITY,
        restitution: 0.3,
        collider: Collider::Convex {
            vertices: vec![
                Vec2::new(-r * 0.866, -r * 0.5), // bottom-left
                Vec2::new(r * 0.866, -r * 0.5),  // bottom-right
                Vec2::new(0.0, r),                // top
            ],
        },
    });
    walls.push(Wall { body: h, label: 'T', fill_color: Color::hex(0x00BFA599) });

    // Octagon — bottom side (circumradius 0.75, CCW via increasing angle).
    let r = 0.75_f32;
    let oct_verts: Vec<Vec2> = (0..8)
        .map(|i| {
            let angle = std::f32::consts::TAU * i as f32 / 8.0;
            Vec2::new(r * angle.cos(), r * angle.sin())
        })
        .collect();
    let h = physics.add_body(Body {
        position: Vec2::new(0.0, -3.0),
        velocity: Vec2::ZERO,
        mass: f32::INFINITY,
        restitution: 0.3,
        collider: Collider::Convex { vertices: oct_verts },
    });
    walls.push(Wall { body: h, label: 'O', fill_color: Color::hex(0xFFD60099) });

    walls
}

pub struct World {
    pub physics: PhysicsWorld,
    pub players: Vec<Player>,
    pub walls: Vec<Wall>,
    pub projectiles: Vec<Projectile>,
    pub camera: Camera,
    start: std::time::Instant,
    last_tick: std::time::Instant,
    pub fps: f32,
    pub cursor_ndc: Vec2,
    input_state: InputState,
}

impl World {
    pub fn new() -> Self {
        let now = std::time::Instant::now();
        let mut world = Self {
            physics: PhysicsWorld::new(),
            players: Vec::new(),
            walls: Vec::new(),
            projectiles: Vec::new(),
            camera: Camera::default(),
            start: now,
            last_tick: now,
            fps: 0.0,
            cursor_ndc: Vec2::ZERO,
            input_state: InputState::default(),
        };
        world
            .players
            .push(Player::new(0, Vec2::ZERO, &mut world.physics));
        world.walls = make_walls(&mut world.physics);
        world
    }

    pub fn tick(&mut self, events: &[InputEvent]) {
        let now = std::time::Instant::now();
        let dt = now.duration_since(self.last_tick).as_secs_f32();
        self.last_tick = now;
        self.fps = self.fps * 0.9 + (1.0 / dt.max(1e-6)) * 0.1;

        for event in events {
            if let InputEvent::CursorMoved { x, y } = event {
                self.cursor_ndc = Vec2::new(*x, *y);
            }
        }

        self.input_state.apply_events(events, self.cursor_ndc);
        let snapshot = self.input_state.snapshot();

        // Collect per-player spawn requests before mutably borrowing physics.
        let mut spawn_requests: Vec<(usize, Vec2, Vec<Vec2>)> = Vec::new();

        for player in &mut self.players {
            let intent = snapshot.player(player.slot);
            let aim_dir = if player.slot == 0 && !intent.aim_from_stick {
                let player_ndc =
                    self.camera.world_to_ndc(self.physics.body(player.body).position);
                let dir = self.cursor_ndc - player_ndc;
                if dir.length_squared() > 1e-6 {
                    dir.normalize()
                } else {
                    player.facing
                }
            } else {
                intent.aim_dir
            };
            if aim_dir.length_squared() > 1e-6 {
                player.facing = aim_dir;
            }
            self.physics.body_mut(player.body).velocity = intent.move_dir * PLAYER_SPEED;

            let volleys = player.weapon.tick(dt, intent.fire);
            if volleys > 0 {
                let player_pos = self.physics.body(player.body).position;
                let dirs = player.weapon.volley_directions(player.facing);
                spawn_requests.push((player.slot, player_pos, dirs));

                // Apply recoil to the player body.
                let recoil = -player.facing * player.weapon.stats.recoil_force;
                self.physics.body_mut(player.body).velocity += recoil;
            }
        }

        for (owner_slot, origin, dirs) in spawn_requests {
            let stats = &self.players[owner_slot].weapon.stats;
            let speed = stats.projectile_speed;
            let size = stats.projectile_size;
            let lifetime = stats.projectile_lifetime;
            let piercing = stats.piercing;

            for dir in dirs {
                let handle = self.physics.add_body(Body {
                    position: origin + dir * (PLAYER_RADIUS_SPAWN_OFFSET + size),
                    velocity: dir * speed,
                    mass: 0.01,
                    restitution: 0.0,
                    collider: Collider::Circle { radius: size },
                });
                self.projectiles.push(Projectile {
                    body: handle,
                    owner_slot,
                    lifetime,
                    piercing,
                    size,
                });
            }
        }

        self.physics.step(dt);

        let live_positions: Vec<Vec2> = self
            .players
            .iter()
            .filter(|p| p.health > 0.0)
            .map(|p| self.physics.body(p.body).position)
            .collect();
        if !live_positions.is_empty() {
            let avg = live_positions.iter().copied().sum::<Vec2>() / live_positions.len() as f32;
            self.camera.update(avg, dt);
        }
        tick_projectiles(&mut self.projectiles, dt);
        cleanup_projectiles(&mut self.projectiles, &mut self.physics, &self.walls);
    }

    pub fn draw(&self, driver: &mut dyn gfx::GraphicsDriver) {
        let backend = driver.backend_name();
        let _elapsed = self.start.elapsed().as_secs_f32();
        driver.clear(GROUND_COLOR);
        draw_walls(&self.walls, &self.physics, driver, &self.camera);
        draw_players(&self.players, &self.physics, driver, &self.camera);
        draw_projectiles(&self.projectiles, &self.physics, driver, &self.camera);

        // Collision HUD: collect which walls the player is touching.
        let contacts = self.physics.contacts();
        let player_body = self.players.first().map(|p| p.body);
        let wall_hits: Vec<(char, bool)> = self
            .walls
            .iter()
            .map(|w| {
                let hit = player_body.map_or(false, |pb| {
                    contacts
                        .iter()
                        .any(|(a, b, _)| (*a == pb && *b == w.body) || (*b == pb && *a == w.body))
                });
                (w.label, hit)
            })
            .collect();

        hud::draw_fps(driver, self.fps);
        hud::draw_backend(driver, backend);
        hud::draw_mouse_pos(driver, self.cursor_ndc);
        hud::draw_collision_hits(driver, &wall_hits);
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

fn draw_projectiles(
    projectiles: &[Projectile],
    physics: &PhysicsWorld,
    driver: &mut dyn gfx::GraphicsDriver,
    camera: &Camera,
) {
    let style = Style {
        fill: Some(Fill::Solid(Color::hex(0xFFFFFFFF))),
        stroke: None,
    };
    for proj in projectiles {
        let pos = physics.body(proj.body).position;
        let ndc = camera.world_to_ndc(pos);
        let r = camera.scale(proj.size);
        draw_shape(driver, &circle(ndc, r), &style, Mat3::IDENTITY);
    }
}

fn draw_walls(
    walls: &[Wall],
    physics: &PhysicsWorld,
    driver: &mut dyn gfx::GraphicsDriver,
    camera: &Camera,
) {
    for wall in walls {
        let body = physics.body(wall.body);
        let pos = body.position;
        let style = Style {
            fill: Some(Fill::Solid(wall.fill_color)),
            stroke: Some(Stroke {
                color: Color::hex(0xFFFFFFCC),
                width: 0.006,
                cap: LineCap::Round,
                join: LineJoin::Round,
            }),
        };
        match &body.collider {
            Collider::Circle { radius } => {
                let ndc = camera.world_to_ndc(pos);
                draw_shape(driver, &circle(ndc, camera.scale(*radius)), &style, Mat3::IDENTITY);
            }
            Collider::Convex { vertices } => {
                let ndc_verts: Vec<Vec2> =
                    vertices.iter().map(|v| camera.world_to_ndc(pos + *v)).collect();
                draw_shape(driver, &polygon(&ndc_verts), &style, Mat3::IDENTITY);
            }
            Collider::Mesh { .. } => {}
        }
    }
}

// ---------------------------------------------------------------------------
// Projectile lifecycle
// ---------------------------------------------------------------------------

fn tick_projectiles(projectiles: &mut Vec<Projectile>, dt: f32) {
    for p in projectiles.iter_mut() {
        p.lifetime -= dt;
    }
}

fn cleanup_projectiles(
    projectiles: &mut Vec<Projectile>,
    physics: &mut PhysicsWorld,
    walls: &[Wall],
) {
    let wall_handles: std::collections::HashSet<BodyHandle> =
        walls.iter().map(|w| w.body).collect();

    let contacts = physics.contacts().to_vec();

    let mut to_remove: Vec<usize> = Vec::new();
    for (idx, proj) in projectiles.iter_mut().enumerate() {
        if proj.lifetime <= 0.0 {
            to_remove.push(idx);
            continue;
        }

        // Check if this projectile hit a wall.
        let hit_wall = contacts.iter().any(|(a, b, _)| {
            let involves_proj =
                *a == proj.body || *b == proj.body;
            let involves_wall =
                wall_handles.contains(a) || wall_handles.contains(b);
            involves_proj && involves_wall
        });

        if hit_wall {
            if proj.piercing == 0 {
                to_remove.push(idx);
            } else {
                proj.piercing -= 1;
            }
        }
    }

    // Remove in reverse index order to keep indices valid.
    to_remove.sort_unstable();
    to_remove.dedup();
    for idx in to_remove.into_iter().rev() {
        let removed = projectiles.swap_remove(idx);
        physics.remove_body(removed.body);
    }
}

fn draw_shape(
    driver: &mut dyn gfx::GraphicsDriver,
    path: &gfx::Path,
    style: &gfx::Style,
    transform: Mat3,
) {
    for mesh in tessellate(path, style) {
        let handle = driver.upload_mesh(&mesh.vertices, &mesh.indices);
        driver.draw_mesh(handle, transform, [1.0, 1.0, 1.0, 1.0]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gfx::driver::GraphicsDriver;
    use gfx_software::SoftwareDriver;
    use input::{
        event::{Axis, Button, InputEvent},
        player::PlayerId,
    };

    #[test]
    fn world_tick_advances_player() {
        let mut world = World::new();
        world.tick(&[InputEvent::Button {
            player: PlayerId::P1,
            button: Button::DPadRight,
            pressed: true,
        }]);
        let pos = world.physics.body(world.players[0].body).position;
        assert!(pos.x > 0.0);
    }

    #[test]
    fn world_tick_applies_expected_player_speed() {
        let mut world = World::new();
        world.tick(&[InputEvent::Button {
            player: PlayerId::P1,
            button: Button::DPadRight,
            pressed: true,
        }]);
        let velocity = world.physics.body(world.players[0].body).velocity;
        assert!((velocity.length() - PLAYER_SPEED).abs() < 1e-5);
    }

    #[test]
    fn releasing_input_stops_player() {
        let mut world = World::new();
        world.tick(&[InputEvent::Button {
            player: PlayerId::P1,
            button: Button::DPadRight,
            pressed: true,
        }]);
        world.tick(&[InputEvent::Button {
            player: PlayerId::P1,
            button: Button::DPadRight,
            pressed: false,
        }]);
        let velocity = world.physics.body(world.players[0].body).velocity;
        assert_eq!(velocity, Vec2::ZERO);
    }

    #[test]
    fn held_dpad_continues_moving_without_repeat_events() {
        let mut world = World::new();
        world.tick(&[InputEvent::Button {
            player: PlayerId::P1,
            button: Button::DPadRight,
            pressed: true,
        }]);
        let x1 = world.physics.body(world.players[0].body).position.x;
        world.tick(&[]);
        let x2 = world.physics.body(world.players[0].body).position.x;
        assert!(x2 > x1);
    }

    #[test]
    fn cursor_updates_player_facing() {
        let mut world = World::new();
        world.tick(&[InputEvent::CursorMoved { x: 0.0, y: 1.0 }]);
        assert_eq!(world.players[0].facing, Vec2::Y);
    }

    /// Regression: aim must point from the player's position to the cursor, not
    /// from the screen centre.  With the player at world (2.5, 0) the player's
    /// NDC position is (0.5, 0) (half_view == 5).  A cursor at NDC (0, 1)
    /// should produce facing = normalize((0, 1) - (0.5, 0)) = normalize((-0.5, 1)).
    /// The old (wrong) code would have produced normalize((0, 1)) = Vec2::Y.
    #[test]
    fn cursor_aim_is_relative_to_player_position() {
        let mut world = World::new();
        // Place the player off-centre at a known world position.
        world.physics.body_mut(world.players[0].body).position = Vec2::new(2.5, 0.0);
        world.tick(&[InputEvent::CursorMoved { x: 0.0, y: 1.0 }]);
        // player NDC = (2.5 / 5.0, 0.0) = (0.5, 0.0)
        // cursor NDC = (0.0, 1.0)
        // expected dir = normalize((0.0, 1.0) - (0.5, 0.0)) = normalize((-0.5, 1.0))
        let expected = Vec2::new(-0.5, 1.0).normalize();
        assert!((world.players[0].facing - expected).length() < 1e-5);
    }

    #[test]
    fn right_stick_overrides_cursor_facing() {
        let mut world = World::new();
        world.tick(&[
            InputEvent::CursorMoved { x: 0.0, y: 1.0 },
            InputEvent::Axis {
                player: PlayerId::P1,
                axis: Axis::RightX,
                value: 1.0,
            },
        ]);
        assert_eq!(world.players[0].facing, Vec2::X);
    }

    #[test]
    fn world_draw_does_not_panic() {
        let mut driver = SoftwareDriver::headless(256, 256);
        let world = World::new();
        driver.begin_frame();
        world.draw(&mut driver);
        driver.end_frame();
    }

    #[test]
    fn world_camera_follows_avg() {
        use crate::player::Player;

        let mut world = World::new();
        // Two players at (-4, 0) and (4, 0) — average is origin.
        world.physics.body_mut(world.players[0].body).position = Vec2::new(-4.0, 0.0);
        world.players.push(Player::new(1, Vec2::new(4.0, 0.0), &mut world.physics));
        // Pre-offset the camera so we can observe it converging toward origin.
        world.camera.position = Vec2::new(0.0, 10.0);

        for _ in 0..100 {
            world.tick(&[]);
        }

        assert!(
            world.camera.position.y < 10.0,
            "camera did not move toward avg: y = {}",
            world.camera.position.y
        );
    }
}
