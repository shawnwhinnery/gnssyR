use glam::{Mat3, Vec2};
use input::InputEvent;
use physics::{narrow, Body, BodyHandle, Collider, Contact, PhysicsWorld};

use rand::Rng;

use crate::{
    actor::{draw_shape, Actor},
    camera::Camera,
    enemy::{dummy::Dummy, Enemy, LootTable},
    hud,
    input::InputState,
    npc::{forgemaster::Forgemaster, FriendlyNpc, NpcKind},
    physics_layers,
    player::{Player, PLAYER_RADIUS},
    scrap::{draw_scrap, Inventory, Scrap, ScrapColor, ScrapShape},
    weapon::{
        perp, rotate, Projectile, ProjectileBehavior, ProjectileMotion, ProjectileOwner,
        ProjectilePhysicsState, ProjectileScriptedState, WeaponStats,
    },
};
use gfx::{
    shape::{circle, line, polygon},
    style::{Fill, LineCap, LineJoin, Stroke, Style},
    Color,
};

pub const GROUND_COLOR: [f32; 4] = [0.13, 0.14, 0.12, 1.0];

const SPAWN_OFFSET: f32 = PLAYER_RADIUS + 0.05;

struct ProjectileSpawnBatch {
    origin: Vec2,
    dirs: Vec<Vec2>,
    owner: ProjectileOwner,
    stats: WeaponStats,
    behavior: ProjectileBehavior,
}

// ---------------------------------------------------------------------------
// Wall
// ---------------------------------------------------------------------------

pub struct Wall {
    pub body: BodyHandle,
    pub label: char,
    pub fill_color: Color,
}

// ---------------------------------------------------------------------------
// World
// ---------------------------------------------------------------------------

pub struct World {
    pub physics: PhysicsWorld,
    pub players: Vec<Player>,
    pub enemies: Vec<Box<dyn Enemy>>,
    pub npcs: Vec<Box<dyn FriendlyNpc>>,
    pub walls: Vec<Wall>,
    pub projectiles: Vec<Projectile>,
    pub scraps: Vec<Scrap>,
    pub inventory: Inventory,
    pub camera: Camera,
    start: std::time::Instant,
    last_tick: std::time::Instant,
    pub fps: f32,
    pub cursor_ndc: Vec2,
    input_state: InputState,
    pub time_scale: f32,
}

impl World {
    /// Create an empty world with one player at the origin and no walls.
    /// Callers add walls via [`add_wall`](Self::add_wall).
    pub fn new() -> Self {
        let now = std::time::Instant::now();
        let mut world = Self {
            physics: PhysicsWorld::new(),
            players: Vec::new(),
            enemies: Vec::new(),
            npcs: Vec::new(),
            walls: Vec::new(),
            projectiles: Vec::new(),
            scraps: Vec::new(),
            inventory: Inventory::new(),
            camera: Camera::default(),
            start: now,
            last_tick: now,
            fps: 0.0,
            cursor_ndc: Vec2::ZERO,
            input_state: InputState::default(),
            time_scale: 1.0,
        };
        world
            .players
            .push(Player::new(0, Vec2::ZERO, &mut world.physics));
        world
    }

    /// Register a physics body as a wall. Returns the body handle so callers
    /// can store it for later removal (e.g. a door that opens).
    pub fn add_wall(&mut self, body: Body, label: char, fill_color: Color) -> BodyHandle {
        let handle = self.physics.add_body(body);
        self.walls.push(Wall {
            body: handle,
            label,
            fill_color,
        });
        handle
    }

    /// Remove a wall body from both the physics world and the wall list.
    pub fn remove_wall(&mut self, handle: BodyHandle) {
        self.walls.retain(|w| w.body != handle);
        self.physics.remove_body(handle);
    }

    pub fn spawn_enemy(&mut self, pos: Vec2) {
        let enemy = Dummy::new(pos, &mut self.physics);
        self.enemies.push(Box::new(enemy));
    }

    pub fn spawn_forgemaster(&mut self, pos: Vec2) {
        self.npcs
            .push(Box::new(Forgemaster::new(pos, &mut self.physics)));
    }

    /// Spawn a scrap directly into the world at the given position.
    pub fn spawn_scrap(&mut self, pos: Vec2, color: ScrapColor, shape: ScrapShape) {
        self.scraps.push(Scrap {
            position: pos,
            color,
            shape,
        });
    }

    /// Return the kind of the nearest friendly NPC within interaction range of P1, if any.
    pub fn nearest_interactable_npc(&self) -> Option<NpcKind> {
        let player_pos = self
            .players
            .first()
            .map(|p| self.physics.body(p.actor.body).position)?;
        self.npcs
            .iter()
            .filter_map(|npc| {
                let npc_pos = self.physics.body(npc.body()).position;
                let dist = player_pos.distance(npc_pos);
                if dist <= npc.interaction_radius() {
                    Some((dist, npc.kind()))
                } else {
                    None
                }
            })
            .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(_, kind)| kind)
    }

    pub fn respawn_player(&mut self, slot: usize) {
        if let Some(player) = self.players.iter_mut().find(|p| p.slot == slot) {
            player.health = 100.0;
            let body = self.physics.body_mut(player.actor.body);
            body.position = Vec2::ZERO;
            body.velocity = Vec2::ZERO;
        }
    }

    pub fn tick(&mut self, events: &[InputEvent]) {
        let now = std::time::Instant::now();
        let real_dt = now.duration_since(self.last_tick).as_secs_f32();
        self.last_tick = now;
        self.fps = self.fps * 0.9 + (1.0 / real_dt.max(1e-6)) * 0.1;
        let dt = real_dt * self.time_scale;

        for event in events {
            if let InputEvent::CursorMoved { x, y } = event {
                self.cursor_ndc = Vec2::new(*x, *y);
            }
        }

        self.input_state.apply_events(events, self.cursor_ndc);
        let snapshot = self.input_state.snapshot();

        // ── player input → spawn batches ───────────────────────────────────────
        let mut spawn_batches: Vec<ProjectileSpawnBatch> = Vec::new();

        for player in &mut self.players {
            if player.health <= 0.0 {
                self.physics.body_mut(player.actor.body).velocity = Vec2::ZERO;
                continue;
            }
            let intent = snapshot.player(player.slot);
            let aim_dir = if player.slot == 0 && !intent.aim_from_stick {
                let player_ndc = self
                    .camera
                    .world_to_ndc(self.physics.body(player.actor.body).position);
                let dir = self.cursor_ndc - player_ndc;
                if dir.length_squared() > 1e-6 {
                    dir.normalize()
                } else {
                    player.actor.facing
                }
            } else {
                intent.aim_dir
            };
            if aim_dir.length_squared() > 1e-6 {
                player.actor.facing = aim_dir;
            }
            self.physics.body_mut(player.actor.body).velocity =
                intent.move_dir * crate::player::PLAYER_SPEED;

            let volleys = player.weapon.tick(dt, intent.fire);
            if volleys > 0 {
                let pos = self.physics.body(player.actor.body).position;
                let dirs = player.weapon.volley_directions(player.actor.facing);
                let stats = player.weapon.stats.clone();
                let behavior = player.weapon.projectile_behavior;
                spawn_batches.push(ProjectileSpawnBatch {
                    origin: pos,
                    dirs,
                    owner: ProjectileOwner::Player(player.slot),
                    stats,
                    behavior,
                });
                let recoil = -player.actor.facing * player.weapon.stats.recoil_force;
                self.physics.body_mut(player.actor.body).velocity += recoil;
            }
        }

        // ── enemy AI → spawn batches ─────────────────────────────────────────
        let player_positions: Vec<Vec2> = self
            .players
            .iter()
            .filter(|p| p.health > 0.0)
            .map(|p| self.physics.body(p.actor.body).position)
            .collect();

        for enemy in &mut self.enemies {
            if !enemy.is_alive() {
                self.physics.body_mut(enemy.body()).velocity = Vec2::ZERO;
                continue;
            }
            let volleys = enemy.tick_ai(dt, &player_positions, &mut self.physics);
            let stats = enemy.weapon_stats().clone();
            let behavior = enemy.projectile_behavior();
            for (origin, dirs) in volleys {
                spawn_batches.push(ProjectileSpawnBatch {
                    origin,
                    dirs,
                    owner: ProjectileOwner::Enemy,
                    stats: stats.clone(),
                    behavior,
                });
            }
        }

        let player_targets: Vec<(BodyHandle, usize, Vec2)> = self
            .players
            .iter()
            .filter(|p| p.health > 0.0)
            .map(|p| {
                let h = p.actor.body;
                let pos = self.physics.body(h).position;
                (h, p.slot, pos)
            })
            .collect();
        let enemy_targets: Vec<(BodyHandle, Vec2)> = self
            .enemies
            .iter()
            .filter(|e| e.is_alive())
            .map(|e| {
                let h = e.body();
                let pos = self.physics.body(h).position;
                (h, pos)
            })
            .collect();

        for batch in spawn_batches {
            for dir in batch.dirs {
                spawn_one_projectile(
                    &mut self.physics,
                    &mut self.projectiles,
                    batch.origin,
                    dir,
                    batch.owner,
                    &batch.stats,
                    batch.behavior,
                    &player_targets,
                    &enemy_targets,
                );
            }
        }

        self.physics.step(dt);

        apply_physics_projectile_friction_and_min_speed(&mut self.projectiles, &mut self.physics, dt);
        update_physics_projectile_wall_bounces(
            &mut self.projectiles,
            &self.walls,
            &self.physics.contacts(),
        );

        let enemy_targets_now: Vec<(BodyHandle, Vec2)> = self
            .enemies
            .iter()
            .filter(|e| e.is_alive())
            .filter_map(|e| {
                let h = e.body();
                self.physics
                    .try_body(h)
                    .map(|b| (h, b.position))
            })
            .collect();
        let player_targets_now: Vec<(BodyHandle, usize, Vec2)> = self
            .players
            .iter()
            .filter(|p| p.health > 0.0)
            .filter_map(|p| {
                let h = p.actor.body;
                self.physics
                    .try_body(h)
                    .map(|b| (h, p.slot, b.position))
            })
            .collect();

        integrate_scripted_projectiles(
            &mut self.projectiles,
            dt,
            &self.physics,
            &enemy_targets_now,
            &player_targets_now,
        );

        let live_positions: Vec<Vec2> = self
            .players
            .iter()
            .filter(|p| p.health > 0.0)
            .map(|p| self.physics.body(p.actor.body).position)
            .collect();
        if !live_positions.is_empty() {
            let avg = live_positions.iter().copied().sum::<Vec2>() / live_positions.len() as f32;
            self.camera.update(avg, dt);
        }

        tick_projectiles(&mut self.projectiles, dt);

        let player_bodies: Vec<(BodyHandle, usize)> = self
            .players
            .iter()
            .map(|p| (p.actor.body, p.slot))
            .collect();
        let enemy_bodies: Vec<BodyHandle> = self.enemies.iter().map(|e| e.body()).collect();
        let contacts = self.physics.contacts().to_vec();

        resolve_damage(
            &self.projectiles,
            &player_bodies,
            &enemy_bodies,
            &contacts,
            &self.physics,
            &mut self.players,
            &mut self.enemies,
        );

        let death_loot = cleanup_dead_enemies(&mut self.enemies, &mut self.physics);
        spawn_scraps(&mut self.scraps, death_loot);
        collect_scraps(
            &mut self.scraps,
            &mut self.inventory,
            &self.players,
            &self.physics,
        );
        let enemy_bodies_live: Vec<BodyHandle> =
            self.enemies.iter().map(|e| e.body()).collect();
        cleanup_projectiles(
            &mut self.projectiles,
            &mut self.physics,
            &self.walls,
            &enemy_bodies_live,
            &player_bodies,
        );
    }

    pub fn draw(&self, driver: &mut dyn gfx::GraphicsDriver) {
        let backend = driver.backend_name();
        let _elapsed = self.start.elapsed().as_secs_f32();
        driver.clear(GROUND_COLOR);
        draw_walls(&self.walls, &self.physics, driver, &self.camera);
        for npc in &self.npcs {
            npc.draw(&self.physics, driver, &self.camera);
        }
        for scrap in &self.scraps {
            draw_scrap(scrap, driver, &self.camera);
        }
        for player in &self.players {
            player.draw(&self.physics, driver, &self.camera);
        }
        for player in &self.players {
            draw_spread_cone(player, &self.physics, driver, &self.camera);
        }
        for enemy in &self.enemies {
            enemy.draw(&self.physics, driver, &self.camera);
        }
        draw_projectiles(&self.projectiles, &self.physics, driver, &self.camera);

        let contacts = self.physics.contacts();
        let player_body = self.players.first().map(|p| p.actor.body);
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

    pub fn alive_enemy_count(&self) -> usize {
        self.enemies.iter().filter(|e| e.is_alive()).count()
    }

    pub fn player_health(&self, slot: usize) -> Option<f32> {
        self.players
            .iter()
            .find(|p| p.slot == slot)
            .map(|p| p.health)
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Projectile spawn & integration
// ---------------------------------------------------------------------------

fn rotate_toward(current: Vec2, target: Vec2, max_angle: f32) -> Vec2 {
    let c = current.normalize_or_zero();
    if c.length_squared() < 1e-12 {
        return target.normalize_or_zero();
    }
    let t = target.normalize_or_zero();
    if t.length_squared() < 1e-12 {
        return c;
    }
    let cross = c.x * t.y - c.y * t.x;
    let dot = c.dot(t).clamp(-1.0, 1.0);
    let angle = cross.atan2(dot);
    let step = angle.clamp(-max_angle, max_angle);
    rotate(c, step).normalize_or_zero()
}

fn acquire_nearest_enemy(from: Vec2, enemies: &[(BodyHandle, Vec2)]) -> Option<BodyHandle> {
    enemies
        .iter()
        .filter_map(|&(h, p)| {
            let d2 = p.distance_squared(from);
            if d2 < 1e-10 {
                return None;
            }
            Some((d2.sqrt(), h))
        })
        .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(_, h)| h)
}

fn acquire_seek_target_player(
    from: Vec2,
    forward: Vec2,
    half_angle: f32,
    enemies: &[(BodyHandle, Vec2)],
) -> Option<BodyHandle> {
    let f = forward.normalize_or_zero();
    if f.length_squared() < 1e-8 {
        return acquire_nearest_enemy(from, enemies);
    }
    let cos_lim = half_angle.cos();
    let mut best_cone: Option<(f32, BodyHandle)> = None;
    for &(h, p) in enemies {
        let v = p - from;
        let d2 = v.length_squared();
        if d2 < 1e-10 {
            continue;
        }
        let d = d2.sqrt();
        let n = v / d;
        if n.dot(f) >= cos_lim {
            if best_cone.map_or(true, |(bd, _)| d < bd) {
                best_cone = Some((d, h));
            }
        }
    }
    if let Some((_, h)) = best_cone {
        return Some(h);
    }
    acquire_nearest_enemy(from, enemies)
}

fn acquire_seek_target_enemy(from: Vec2, players: &[(BodyHandle, usize, Vec2)]) -> Option<BodyHandle> {
    players
        .iter()
        .filter_map(|&(h, _slot, p)| {
            let d2 = p.distance_squared(from);
            if d2 < 1e-10 {
                return None;
            }
            Some((d2.sqrt(), h))
        })
        .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(_, h)| h)
}

fn spawn_one_projectile(
    physics: &mut PhysicsWorld,
    projectiles: &mut Vec<Projectile>,
    origin: Vec2,
    dir_volley: Vec2,
    owner: ProjectileOwner,
    stats: &WeaponStats,
    behavior: ProjectileBehavior,
    player_targets: &[(BodyHandle, usize, Vec2)],
    enemy_targets: &[(BodyHandle, Vec2)],
) {
    let size = stats.projectile_size;
    let aim = if dir_volley.length_squared() > 1e-8 {
        dir_volley.normalize()
    } else {
        Vec2::X
    };
    let muzzle = origin + aim * (SPAWN_OFFSET + size);

    let lifetime = stats.projectile_lifetime;
    let piercing = stats.piercing;
    let damage = stats.damage_total;
    let collider = Collider::Circle { radius: size };

    match behavior {
        ProjectileBehavior::Physics => {
            let (collision_layers, collision_mask) = match owner {
                ProjectileOwner::Player(_) => physics_layers::projectile_player_owned(),
                ProjectileOwner::Enemy => physics_layers::projectile_enemy_owned(),
            };
            let handle = physics.add_body(Body {
                position: muzzle,
                velocity: aim * stats.projectile_speed,
                mass: 0.01,
                restitution: 0.82,
                collision_layers,
                collision_mask,
                collider,
            });
            projectiles.push(Projectile {
                motion: ProjectileMotion::Physics(ProjectilePhysicsState {
                    body: handle,
                    bounce_count: 0,
                    was_touching_wall: false,
                }),
                owner,
                lifetime,
                piercing,
                size,
                damage,
                stats: stats.clone(),
            });
        }
        ProjectileBehavior::Bullet
        | ProjectileBehavior::Rocket
        | ProjectileBehavior::Oscillating
        | ProjectileBehavior::Seeking => {
            let speed = stats.projectile_speed;
            let seek_target = if behavior == ProjectileBehavior::Seeking {
                match owner {
                    ProjectileOwner::Player(_) => acquire_seek_target_player(
                        muzzle,
                        aim,
                        stats.seeking_acquire_half_angle,
                        enemy_targets,
                    ),
                    ProjectileOwner::Enemy => acquire_seek_target_enemy(muzzle, player_targets),
                }
            } else {
                None
            };
            let st = ProjectileScriptedState {
                position: muzzle,
                dir: aim,
                speed,
                phase_time: 0.0,
                distance_along: 0.0,
                anchor: muzzle,
                last_enemy_body: None,
                last_player_slot: None,
                seek_target,
            };
            projectiles.push(Projectile {
                motion: ProjectileMotion::Scripted {
                    behavior,
                    state: st,
                },
                owner,
                lifetime,
                piercing,
                size,
                damage,
                stats: stats.clone(),
            });
        }
    }
}

fn integrate_scripted_projectiles(
    projectiles: &mut [Projectile],
    dt: f32,
    physics: &PhysicsWorld,
    enemy_targets: &[(BodyHandle, Vec2)],
    player_targets: &[(BodyHandle, usize, Vec2)],
) {
    use std::f32::consts::TAU;
    for proj in projectiles.iter_mut() {
        let ProjectileMotion::Scripted {
            behavior,
            state,
        } = &mut proj.motion
        else {
            continue;
        };
        let stats = &proj.stats;
        match *behavior {
            ProjectileBehavior::Bullet => {
                state.position += state.dir * state.speed * dt;
            }
            ProjectileBehavior::Rocket => {
                state.speed += stats.rocket_acceleration * dt;
                state.position += state.dir * state.speed * dt;
            }
            ProjectileBehavior::Oscillating => {
                state.phase_time += dt;
                state.distance_along += stats.projectile_speed * dt;
                let lateral = perp(state.dir)
                    * stats.oscillation_magnitude
                    * (TAU * stats.oscillation_frequency * state.phase_time).sin();
                state.position = state.anchor + state.dir * state.distance_along + lateral;
            }
            ProjectileBehavior::Seeking => {
                let turn_radius = stats.seeking_turn_radius.max(0.05);
                let w_max = state.speed / turn_radius;
                let max_turn = w_max * dt;

                let mut target_pos: Option<Vec2> = None;
                if let Some(th) = state.seek_target {
                    if let Some(tb) = physics.try_body(th) {
                        target_pos = Some(tb.position);
                    } else {
                        state.seek_target = None;
                    }
                }
                if target_pos.is_none() {
                    state.seek_target = match proj.owner {
                        ProjectileOwner::Player(_) => acquire_seek_target_player(
                            state.position,
                            state.dir,
                            stats.seeking_acquire_half_angle,
                            enemy_targets,
                        ),
                        ProjectileOwner::Enemy => {
                            acquire_seek_target_enemy(state.position, player_targets)
                        }
                    };
                    if let Some(th) = state.seek_target {
                        target_pos = physics.try_body(th).map(|b| b.position);
                    }
                }
                if let Some(tp) = target_pos {
                    let to = tp - state.position;
                    if to.length_squared() > 1e-10 {
                        let want = to.normalize();
                        state.dir = rotate_toward(state.dir, want, max_turn);
                    }
                }
                state.position += state.dir * state.speed * dt;
            }
            ProjectileBehavior::Physics => {}
        }
    }
}

fn apply_physics_projectile_friction_and_min_speed(
    projectiles: &mut [Projectile],
    physics: &mut PhysicsWorld,
    dt: f32,
) {
    for proj in projectiles.iter_mut() {
        let ProjectileMotion::Physics(phys) = &mut proj.motion else {
            continue;
        };
        let stats = &proj.stats;
        let b = physics.body_mut(phys.body);
        let k = stats.physics_friction.max(0.0);
        b.velocity *= (-k * dt).exp();
        let s = b.velocity.length();
        if s < stats.physics_min_speed {
            proj.lifetime = 0.0;
        }
    }
}

fn update_physics_projectile_wall_bounces(
    projectiles: &mut [Projectile],
    walls: &[Wall],
    contacts: &[(BodyHandle, BodyHandle, Contact)],
) {
    let wall_handles: std::collections::HashSet<BodyHandle> =
        walls.iter().map(|w| w.body).collect();
    for proj in projectiles.iter_mut() {
        let ProjectileMotion::Physics(phys) = &mut proj.motion else {
            continue;
        };
        let max_b = proj.stats.physics_max_bounces;
        let touching = contacts.iter().any(|(a, b, _)| {
            let involves_proj = *a == phys.body || *b == phys.body;
            let involves_wall = wall_handles.contains(a) || wall_handles.contains(b);
            involves_proj && involves_wall
        });
        if max_b == 0 {
            phys.was_touching_wall = touching;
            continue;
        }
        if touching && !phys.was_touching_wall {
            phys.bounce_count = phys.bounce_count.saturating_add(1);
            if phys.bounce_count >= max_b {
                proj.lifetime = 0.0;
            }
        }
        phys.was_touching_wall = touching;
    }
}

fn circle_overlaps_body(proj_pos: Vec2, radius: f32, body: &Body) -> bool {
    narrow::detect(
        proj_pos,
        &Collider::Circle { radius },
        body.position,
        &body.collider,
    )
    .is_some()
}

// ---------------------------------------------------------------------------
// Damage resolution
// ---------------------------------------------------------------------------

fn resolve_damage(
    projectiles: &[Projectile],
    player_bodies: &[(BodyHandle, usize)],
    enemy_bodies: &[BodyHandle],
    contacts: &[(BodyHandle, BodyHandle, Contact)],
    physics: &PhysicsWorld,
    players: &mut Vec<Player>,
    enemies: &mut Vec<Box<dyn Enemy>>,
) {
    for proj in projectiles {
        let dmg = proj.rocket_impact_damage();
        match proj.owner {
            ProjectileOwner::Enemy => {
                for &(pb, slot) in player_bodies {
                    let hit = match &proj.motion {
                        ProjectileMotion::Physics(s) => contacts.iter().any(|(a, b, _)| {
                            (*a == s.body && *b == pb) || (*b == s.body && *a == pb)
                        }),
                        ProjectileMotion::Scripted { state, .. } => {
                            if let Some(body) = physics.try_body(pb) {
                                circle_overlaps_body(state.position, proj.size, body)
                            } else {
                                false
                            }
                        }
                    };
                    if hit {
                        if let Some(player) = players.iter_mut().find(|p| p.slot == slot) {
                            player.health = (player.health - dmg).max(0.0);
                        }
                    }
                }
            }
            ProjectileOwner::Player(_) => {
                for (i, &eb) in enemy_bodies.iter().enumerate() {
                    let hit = match &proj.motion {
                        ProjectileMotion::Physics(s) => contacts.iter().any(|(a, b, _)| {
                            (*a == s.body && *b == eb) || (*b == s.body && *a == eb)
                        }),
                        ProjectileMotion::Scripted { state, .. } => {
                            if let Some(body) = physics.try_body(eb) {
                                circle_overlaps_body(state.position, proj.size, body)
                            } else {
                                false
                            }
                        }
                    };
                    if hit {
                        if let Some(enemy) = enemies.get_mut(i) {
                            enemy.take_damage(dmg);
                        }
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Dead enemy cleanup
// ---------------------------------------------------------------------------

fn cleanup_dead_enemies(
    enemies: &mut Vec<Box<dyn Enemy>>,
    physics: &mut PhysicsWorld,
) -> Vec<(Vec2, LootTable)> {
    let mut loot = Vec::new();
    let mut i = 0;
    while i < enemies.len() {
        if !enemies[i].is_alive() {
            let body = enemies[i].body();
            let pos = physics.body(body).position;
            loot.push((pos, enemies[i].loot_table()));
            physics.remove_body(body);
            enemies.swap_remove(i);
        } else {
            i += 1;
        }
    }
    loot
}

fn spawn_scraps(scraps: &mut Vec<Scrap>, death_loot: Vec<(Vec2, LootTable)>) {
    use rand::seq::SliceRandom;
    let mut rng = rand::thread_rng();

    for (pos, table) in death_loot {
        let count = rng.gen_range(table.min_drops..=table.max_drops);
        for _ in 0..count {
            let color = *ScrapColor::ALL.choose(&mut rng).unwrap();
            let shape = *ScrapShape::ALL.choose(&mut rng).unwrap();
            let offset = Vec2::new(rng.gen_range(-0.3..=0.3), rng.gen_range(-0.3..=0.3));
            scraps.push(Scrap {
                position: pos + offset,
                color,
                shape,
            });
        }
    }
}

fn collect_scraps(
    scraps: &mut Vec<Scrap>,
    inventory: &mut Inventory,
    players: &[Player],
    physics: &PhysicsWorld,
) {
    const COLLECT_RADIUS: f32 = PLAYER_RADIUS + 0.18;
    let player_positions: Vec<Vec2> = players
        .iter()
        .filter(|p| p.health > 0.0)
        .map(|p| physics.body(p.actor.body).position)
        .collect();

    let mut i = 0;
    while i < scraps.len() {
        let collected = player_positions
            .iter()
            .any(|&pp| pp.distance(scraps[i].position) < COLLECT_RADIUS);
        if collected {
            let s = scraps.swap_remove(i);
            inventory.add(s.color, s.shape);
        } else {
            i += 1;
        }
    }
}

// ---------------------------------------------------------------------------
// Drawing helpers
// ---------------------------------------------------------------------------

fn rotate_vec(v: Vec2, angle: f32) -> Vec2 {
    let (sin, cos) = angle.sin_cos();
    Vec2::new(v.x * cos - v.y * sin, v.x * sin + v.y * cos)
}

fn draw_spread_cone(
    player: &Player,
    physics: &PhysicsWorld,
    driver: &mut dyn gfx::GraphicsDriver,
    camera: &Camera,
) {
    let stats = &player.weapon.stats;
    let half_angle = stats.shot_arc / 2.0 + stats.jitter + player.weapon.kickback;
    if half_angle < 1e-4 {
        return;
    }
    let pos = physics.body(player.actor.body).position;
    let range = (stats.projectile_speed * stats.projectile_lifetime).min(8.0);
    let ndc_pos = camera.world_to_ndc(pos);
    let style = Style::stroked(Stroke {
        color: Color::hex(0xFFFFFF44),
        width: 0.003,
        cap: LineCap::Round,
        join: LineJoin::Round,
    });
    let left_end = camera.world_to_ndc(pos + rotate_vec(player.actor.facing, half_angle) * range);
    let right_end = camera.world_to_ndc(pos + rotate_vec(player.actor.facing, -half_angle) * range);
    draw_shape(driver, &line(ndc_pos, left_end), &style, Mat3::IDENTITY);
    draw_shape(driver, &line(ndc_pos, right_end), &style, Mat3::IDENTITY);
}

fn draw_projectiles(
    projectiles: &[Projectile],
    physics: &PhysicsWorld,
    driver: &mut dyn gfx::GraphicsDriver,
    camera: &Camera,
) {
    let player_style = Style {
        fill: Some(Fill::Solid(Color::hex(0xFFFFFFFF))),
        stroke: None,
    };
    let enemy_style = Style {
        fill: Some(Fill::Solid(Color::hex(0xFF6666FF))),
        stroke: None,
    };
    for proj in projectiles {
        let pos = proj.world_position(physics);
        let ndc = camera.world_to_ndc(pos);
        let r = camera.scale(proj.size);
        let style = match proj.owner {
            ProjectileOwner::Enemy => &enemy_style,
            ProjectileOwner::Player(_) => &player_style,
        };
        draw_shape(driver, &circle(ndc, r), style, Mat3::IDENTITY);
    }
}

pub fn draw_walls(
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
                draw_shape(
                    driver,
                    &circle(ndc, camera.scale(*radius)),
                    &style,
                    Mat3::IDENTITY,
                );
            }
            Collider::Convex { vertices } => {
                let ndc_verts: Vec<Vec2> = vertices
                    .iter()
                    .map(|v| camera.world_to_ndc(pos + *v))
                    .collect();
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
    enemy_bodies: &[BodyHandle],
    player_bodies: &[(BodyHandle, usize)],
) {
    let mut to_remove: Vec<usize> = Vec::new();
    for (idx, proj) in projectiles.iter_mut().enumerate() {
        if proj.lifetime <= 0.0 {
            to_remove.push(idx);
            continue;
        }

        match &mut proj.motion {
            ProjectileMotion::Physics(_) => {
                // Physics: TTL, min-speed, and bounce cap only (handled elsewhere).
            }
            ProjectileMotion::Scripted { state, .. } => {
                let mut wall_hit = false;
                for w in walls {
                    let body = physics.body(w.body);
                    if circle_overlaps_body(state.position, proj.size, body) {
                        wall_hit = true;
                        break;
                    }
                }
                if wall_hit {
                    to_remove.push(idx);
                    continue;
                }

                match proj.owner {
                    ProjectileOwner::Player(_) => {
                        let mut overlapping_h: Option<BodyHandle> = None;
                        for &eb in enemy_bodies {
                            if let Some(body) = physics.try_body(eb) {
                                if circle_overlaps_body(state.position, proj.size, body) {
                                    overlapping_h = Some(eb);
                                    break;
                                }
                            }
                        }
                        let entering = match (overlapping_h, state.last_enemy_body) {
                            (Some(h), None) => Some(h),
                            (Some(h), Some(p)) if h != p => Some(h),
                            _ => None,
                        };
                        if entering.is_some() {
                            if proj.piercing == 0 {
                                to_remove.push(idx);
                            } else {
                                proj.piercing -= 1;
                            }
                        }
                        state.last_enemy_body = overlapping_h;
                    }
                    ProjectileOwner::Enemy => {
                        let mut overlapping_slot: Option<usize> = None;
                        for &(pb, slot) in player_bodies {
                            if let Some(body) = physics.try_body(pb) {
                                if circle_overlaps_body(state.position, proj.size, body) {
                                    overlapping_slot = Some(slot);
                                    break;
                                }
                            }
                        }
                        let entering = match (overlapping_slot, state.last_player_slot) {
                            (Some(s), None) => Some(s),
                            (Some(s), Some(p)) if s != p => Some(s),
                            _ => None,
                        };
                        if entering.is_some() {
                            if proj.piercing == 0 {
                                to_remove.push(idx);
                            } else {
                                proj.piercing -= 1;
                            }
                        }
                        state.last_player_slot = overlapping_slot;
                    }
                }
            }
        }
    }

    to_remove.sort_unstable();
    to_remove.dedup();
    for idx in to_remove.into_iter().rev() {
        let removed = projectiles.swap_remove(idx);
        if let Some(h) = removed.physics_body() {
            physics.remove_body(h);
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

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
        let pos = world.physics.body(world.players[0].actor.body).position;
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
        let velocity = world.physics.body(world.players[0].actor.body).velocity;
        assert!((velocity.length() - crate::player::PLAYER_SPEED).abs() < 1e-5);
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
        let velocity = world.physics.body(world.players[0].actor.body).velocity;
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
        let x1 = world.physics.body(world.players[0].actor.body).position.x;
        world.tick(&[]);
        let x2 = world.physics.body(world.players[0].actor.body).position.x;
        assert!(x2 > x1);
    }

    #[test]
    fn cursor_updates_player_facing() {
        let mut world = World::new();
        world.tick(&[InputEvent::CursorMoved { x: 0.0, y: 1.0 }]);
        assert_eq!(world.players[0].actor.facing, Vec2::Y);
    }

    /// Regression: aim must point from the player's position to the cursor, not
    /// from the screen centre.
    #[test]
    fn cursor_aim_is_relative_to_player_position() {
        use crate::camera::HALF_VIEW;
        let mut world = World::new();
        let player_x = 2.5_f32;
        world.physics.body_mut(world.players[0].actor.body).position = Vec2::new(player_x, 0.0);
        world.tick(&[InputEvent::CursorMoved { x: 0.0, y: 1.0 }]);
        let player_ndc_x = player_x / HALF_VIEW;
        let expected = Vec2::new(-player_ndc_x, 1.0).normalize();
        assert!(
            (world.players[0].actor.facing - expected).length() < 1e-5,
            "facing {:?} expected {:?}",
            world.players[0].actor.facing,
            expected
        );
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
        assert_eq!(world.players[0].actor.facing, Vec2::X);
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
        world.physics.body_mut(world.players[0].actor.body).position = Vec2::new(-4.0, 0.0);
        world
            .players
            .push(Player::new(1, Vec2::new(4.0, 0.0), &mut world.physics));
        world.camera.position = Vec2::new(0.0, 10.0);

        // `World::tick` uses wall-clock `Instant` deltas; a tight loop yields ~0 dt unless
        // some real time elapses before the first tick.
        std::thread::sleep(std::time::Duration::from_millis(12));
        for _ in 0..100 {
            world.tick(&[]);
        }

        assert!(
            world.camera.position.y < 10.0,
            "camera did not move toward avg: y = {}",
            world.camera.position.y
        );
    }

    #[test]
    fn spawn_enemy_adds_to_enemies() {
        let mut world = World::new();
        assert_eq!(world.enemies.len(), 0);
        world.spawn_enemy(Vec2::new(2.0, 0.0));
        assert_eq!(world.enemies.len(), 1);
        assert!(world.enemies[0].is_alive());
    }

    #[test]
    fn respawn_player_resets_health_and_position() {
        let mut world = World::new();
        world.players[0].health = 0.0;
        world.physics.body_mut(world.players[0].actor.body).position = Vec2::new(5.0, 5.0);
        world.respawn_player(0);
        assert_eq!(world.players[0].health, 100.0);
        let pos = world.physics.body(world.players[0].actor.body).position;
        assert_eq!(pos, Vec2::ZERO);
    }

    #[test]
    fn enemy_is_removed_when_health_reaches_zero() {
        let mut world = World::new();
        world.spawn_enemy(Vec2::new(2.0, 0.0));
        world.enemies[0].take_damage(1000.0);
        assert!(!world.enemies[0].is_alive());
        world.tick(&[]);
        assert_eq!(world.enemies.len(), 0);
    }

    #[test]
    fn add_wall_registers_in_walls_and_physics() {
        let mut world = World::new();
        assert_eq!(world.walls.len(), 0);
        let (collision_layers, collision_mask) = crate::physics_layers::wall_collision();
        world.add_wall(
            Body {
                position: Vec2::new(3.0, 0.0),
                velocity: Vec2::ZERO,
                mass: f32::INFINITY,
                restitution: 0.3,
                collision_layers,
                collision_mask,
                collider: physics::Collider::Circle { radius: 0.5 },
            },
            'T',
            Color::hex(0xFFFFFFFF),
        );
        assert_eq!(world.walls.len(), 1);
    }

    #[test]
    fn remove_wall_clears_from_walls_and_physics() {
        let mut world = World::new();
        let (collision_layers, collision_mask) = crate::physics_layers::wall_collision();
        let h = world.add_wall(
            Body {
                position: Vec2::new(3.0, 0.0),
                velocity: Vec2::ZERO,
                mass: f32::INFINITY,
                restitution: 0.3,
                collision_layers,
                collision_mask,
                collider: physics::Collider::Circle { radius: 0.5 },
            },
            'T',
            Color::hex(0xFFFFFFFF),
        );
        world.remove_wall(h);
        assert_eq!(world.walls.len(), 0);
    }
}
