use glam::Vec2;
use physics::{BodyHandle, PhysicsWorld};
use rand::Rng as _;

// ---------------------------------------------------------------------------
// Projectile behavior (weapon selection + per-shot motion kind)
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ProjectileBehavior {
    /// Straight line at fixed speed; cheap circle-vs-geometry tests (no rigid body).
    Bullet,
    /// Full rigid-body simulation until TTL, min speed, or max wall bounces.
    #[default]
    Physics,
    /// Accelerates along spawn direction; damage scales with speed.
    Rocket,
    /// Forward motion with perpendicular sine offset (frequency / magnitude from stats).
    Oscillating,
    /// Fired along aim; turns toward a tracked target with limited turn rate (`seeking_turn_radius`).
    Seeking,
}

impl ProjectileBehavior {
    pub const ALL: [ProjectileBehavior; 5] = [
        ProjectileBehavior::Bullet,
        ProjectileBehavior::Physics,
        ProjectileBehavior::Rocket,
        ProjectileBehavior::Oscillating,
        ProjectileBehavior::Seeking,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Bullet => "Bullet",
            Self::Physics => "Physics",
            Self::Rocket => "Rocket",
            Self::Oscillating => "Oscillating",
            Self::Seeking => "Seeking",
        }
    }
}

// ---------------------------------------------------------------------------
// Stats
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct WeaponStats {
    /// Rounds per second (governs the cooldown between firing cycles).
    pub fire_rate: f32,
    /// Number of projectiles spawned simultaneously per shot.
    pub projectiles_per_shot: u32,
    /// Total angular spread in radians for multi-projectile shots.
    /// 0.0 = all projectiles travel in exactly the same direction.
    pub shot_arc: f32,
    /// Number of shots fired per trigger pull (1 = semi-auto).
    pub burst_count: u32,
    /// Time in seconds between rounds within a burst.
    pub burst_delay: f32,
    /// Max random angular variance per projectile in radians (0.0 = perfect accuracy).
    pub jitter: f32,
    /// Extra spread (radians) added to runtime `Weapon::kickback` per volley (full auto / burst).
    pub kickback: f32,
    /// Stability time constant in seconds: kickback decays as exp(-dt/stability) every tick.
    pub stability: f32,
    /// Muzzle velocity of each projectile (world units per second).
    /// For `Rocket`, this is the initial speed; acceleration adds on top.
    pub projectile_speed: f32,
    /// Collision radius of each projectile.
    pub projectile_size: f32,
    /// Maximum time in seconds a projectile lives before despawning.
    pub projectile_lifetime: f32,
    /// For non-physics: extra enemy hits before despawn (0 = despawn on first enemy hit).
    /// Walls always stop non-physics projectiles. Ignored for `ProjectileBehavior::Physics`.
    pub piercing: u32,
    /// Flat base damage per projectile hit.
    pub damage_total: f32,
    /// Backward impulse applied to the owner body when a shot is fired.
    pub recoil_force: f32,

    // --- Oscillating ---
    /// Lateral oscillation frequency in Hz (`sin(τ * f * t)`).
    pub oscillation_frequency: f32,
    /// Lateral offset amplitude in world units.
    pub oscillation_magnitude: f32,

    // --- Physics projectile ---
    /// 0 = unlimited wall bounces; otherwise despawn after this many wall touch events.
    pub physics_max_bounces: u32,
    /// Linear speed damping per second (higher = faster slowdown): `v *= exp(-k * dt)`.
    pub physics_friction: f32,
    /// Despawn when speed falls below this (world units/s).
    pub physics_min_speed: f32,

    // --- Rocket ---
    /// Speed increase per second along the flight direction.
    pub rocket_acceleration: f32,
    /// Extra damage per unit speed on impact (added to `damage_total`).
    pub kinetic_damage_scale: f32,

    // --- Seeking ---
    /// Minimum turn radius in world units; max turn rate ≈ `speed / radius` (rad/s).
    pub seeking_turn_radius: f32,
    /// Half-angle of the forward acquisition cone in radians (total cone = 2× this).
    pub seeking_acquire_half_angle: f32,
}

impl Default for WeaponStats {
    fn default() -> Self {
        Self {
            fire_rate: 5.0,
            projectiles_per_shot: 1,
            shot_arc: 0.0,
            burst_count: 1,
            burst_delay: 0.05,
            jitter: 0.0,
            // ~1° per volley (defaults tuned for visible full-auto spread; no hard cap).
            kickback: 1.0_f32.to_radians(),
            stability: 0.25,
            projectile_speed: 15.0,
            projectile_size: 0.08,
            projectile_lifetime: 2.0,
            piercing: 0,
            damage_total: 10.0,
            recoil_force: 0.5,
            oscillation_frequency: 2.0,
            oscillation_magnitude: 0.15,
            physics_max_bounces: 0,
            physics_friction: 0.35,
            physics_min_speed: 0.4,
            rocket_acceleration: 12.0,
            kinetic_damage_scale: 0.35,
            seeking_turn_radius: 2.5,
            seeking_acquire_half_angle: std::f32::consts::FRAC_PI_2,
        }
    }
}

// ---------------------------------------------------------------------------
// State machine
// ---------------------------------------------------------------------------

pub enum WeaponFiringState {
    /// Ready to accept a fire intent.
    Idle,
    /// Waiting `f32` seconds before returning to Idle.
    Cooldown(f32),
    /// Mid-burst: `remaining` volleys left, `next_time` seconds until the next one.
    Burst { remaining: u32, next_time: f32 },
    /// Waiting `f32` seconds for a reload to complete.
    Reloading(f32),
}

// ---------------------------------------------------------------------------
// Weapon
// ---------------------------------------------------------------------------

pub struct Weapon {
    pub stats: WeaponStats,
    pub state: WeaponFiringState,
    /// Accumulated sustained-fire spread in radians (extra bloom; decays via `WeaponStats::stability`).
    pub kickback: f32,
    /// How newly spawned projectiles integrate and collide.
    pub projectile_behavior: ProjectileBehavior,
}

impl WeaponFiringState {
    /// Human-readable label for the current state (for HUD display).
    pub fn label(&self) -> &'static str {
        match self {
            Self::Idle => "READY",
            Self::Cooldown(_) => "COOLDOWN",
            Self::Burst { .. } => "BURST",
            Self::Reloading(_) => "RELOAD",
        }
    }

    /// Remaining time in the current timed state, or 0.0 for Idle / Burst.
    pub fn remaining_secs(&self) -> f32 {
        match self {
            Self::Cooldown(t) | Self::Reloading(t) => t.max(0.0),
            Self::Burst { next_time, .. } => next_time.max(0.0),
            Self::Idle => 0.0,
        }
    }
}

impl Weapon {
    pub fn new(stats: WeaponStats) -> Self {
        Self {
            stats,
            state: WeaponFiringState::Idle,
            kickback: 0.0,
            projectile_behavior: ProjectileBehavior::default(),
        }
    }

    /// Advance the weapon state machine by `dt` seconds.
    ///
    /// Returns the number of projectile volleys to spawn this tick.
    /// Each volley should spawn `stats.projectiles_per_shot` projectiles.
    pub fn tick(&mut self, dt: f32, fire_intent: bool) -> u32 {
        let mut shots = 0u32;

        match &mut self.state {
            WeaponFiringState::Idle => {
                if fire_intent {
                    shots = 1;
                    if self.stats.burst_count > 1 {
                        self.state = WeaponFiringState::Burst {
                            remaining: self.stats.burst_count - 1,
                            next_time: self.stats.burst_delay,
                        };
                    } else {
                        self.state = WeaponFiringState::Cooldown(1.0 / self.stats.fire_rate);
                    }
                }
            }

            WeaponFiringState::Cooldown(ref mut t) => {
                *t -= dt;
                if *t <= 0.0 {
                    if fire_intent {
                        shots = 1;
                        if self.stats.burst_count > 1 {
                            self.state = WeaponFiringState::Burst {
                                remaining: self.stats.burst_count - 1,
                                next_time: self.stats.burst_delay,
                            };
                        } else {
                            *t = 1.0 / self.stats.fire_rate;
                        }
                    } else {
                        self.state = WeaponFiringState::Idle;
                    }
                }
            }

            WeaponFiringState::Burst {
                ref mut remaining,
                ref mut next_time,
            } => {
                *next_time -= dt;
                if *next_time <= 0.0 {
                    shots = 1;
                    *remaining -= 1;
                    if *remaining == 0 {
                        self.state = WeaponFiringState::Cooldown(1.0 / self.stats.fire_rate);
                    } else {
                        *next_time += self.stats.burst_delay;
                    }
                }
            }

            WeaponFiringState::Reloading(ref mut t) => {
                *t -= dt;
                if *t <= 0.0 {
                    self.state = WeaponFiringState::Idle;
                }
            }
        }

        if self.kickback > 0.0 {
            let tau = self.stats.stability.max(1e-4);
            self.kickback *= (-dt / tau).exp();
            if self.kickback < 1e-6 {
                self.kickback = 0.0;
            }
        }

        if shots > 0 {
            self.kickback += self.stats.kickback;
        }

        shots
    }

    /// Direction vectors for all projectiles in one volley, given a `facing` direction.
    ///
    /// Spreads `projectiles_per_shot` directions evenly across `shot_arc` radians,
    /// centred on `facing`.
    pub fn volley_directions(&self, facing: Vec2) -> Vec<Vec2> {
        let n = self.stats.projectiles_per_shot;
        if n == 0 {
            return vec![];
        }
        let mut rng = rand::thread_rng();
        let spread = self.stats.jitter + self.kickback;

        let mut jitter = || -> f32 {
            if spread > 0.0 {
                rng.gen_range(-spread..spread)
            } else {
                0.0
            }
        };

        if n == 1 || self.stats.shot_arc == 0.0 {
            return vec![rotate(facing, jitter())];
        }

        let half = self.stats.shot_arc / 2.0;
        let step = self.stats.shot_arc / (n - 1) as f32;
        (0..n)
            .map(|i| {
                let angle = -half + step * i as f32;
                rotate(facing, angle + jitter())
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Projectile
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ProjectileOwner {
    Player(usize),
    Enemy,
}

/// Rigid-body projectile tracked by [`physics::PhysicsWorld`].
#[derive(Clone)]
pub struct ProjectilePhysicsState {
    pub body: BodyHandle,
    pub bounce_count: u32,
    pub was_touching_wall: bool,
}

/// Kinematic projectile: circle overlap tests only (no rigid body).
#[derive(Clone)]
pub struct ProjectileScriptedState {
    pub position: Vec2,
    pub dir: Vec2,
    /// Scalar speed along `dir` (for rocket acceleration).
    pub speed: f32,
    /// Time since spawn (for oscillation phase).
    pub phase_time: f32,
    /// Integrated distance along `dir` (for oscillating forward component).
    pub distance_along: f32,
    /// Muzzle position fixed at spawn (`Oscillating` path is rebuilt from this).
    pub anchor: Vec2,
    /// Last enemy body overlapped (piercing / one hit per target).
    pub last_enemy_body: Option<BodyHandle>,
    /// Last player slot overlapped (enemy-owned shots).
    pub last_player_slot: Option<usize>,
    /// Enemy body being tracked (`Seeking`); cleared when invalid or lost.
    pub seek_target: Option<BodyHandle>,
}

#[derive(Clone)]
pub enum ProjectileMotion {
    Physics(ProjectilePhysicsState),
    Scripted {
        behavior: ProjectileBehavior,
        state: ProjectileScriptedState,
    },
}

pub struct Projectile {
    pub motion: ProjectileMotion,
    pub owner: ProjectileOwner,
    /// Remaining seconds before the projectile despawns due to age.
    pub lifetime: f32,
    /// For scripted: enemy pierce allowance. Ignored for physics (see cleanup rules).
    pub piercing: u32,
    /// Visual/collision radius (world units).
    pub size: f32,
    /// Base damage (rocket adds `kinetic_damage_scale * speed` on hit).
    pub damage: f32,
    /// Tunables copied at spawn so mid-flight shots keep consistent parameters.
    pub stats: WeaponStats,
}

impl Projectile {
    pub fn behavior_kind(&self) -> ProjectileBehavior {
        match &self.motion {
            ProjectileMotion::Physics(_) => ProjectileBehavior::Physics,
            ProjectileMotion::Scripted { behavior, .. } => *behavior,
        }
    }

    pub fn world_position(&self, physics: &PhysicsWorld) -> Vec2 {
        match &self.motion {
            ProjectileMotion::Physics(s) => physics.body(s.body).position,
            ProjectileMotion::Scripted { state, .. } => state.position,
        }
    }

    pub fn physics_body(&self) -> Option<BodyHandle> {
        match &self.motion {
            ProjectileMotion::Physics(s) => Some(s.body),
            ProjectileMotion::Scripted { .. } => None,
        }
    }

    pub fn rocket_impact_damage(&self) -> f32 {
        match &self.motion {
            ProjectileMotion::Scripted {
                behavior: ProjectileBehavior::Rocket,
                state,
            } => self.damage + self.stats.kinetic_damage_scale * state.speed,
            _ => self.damage,
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Rotate a 2-D vector by `angle` radians.
pub(crate) fn rotate(v: Vec2, angle: f32) -> Vec2 {
    let (sin, cos) = angle.sin_cos();
    Vec2::new(v.x * cos - v.y * sin, v.x * sin + v.y * cos)
}

/// Unit vector perpendicular to `dir` (90° CCW).
pub(crate) fn perp(dir: Vec2) -> Vec2 {
    Vec2::new(-dir.y, dir.x)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_stats() -> WeaponStats {
        WeaponStats {
            fire_rate: 10.0,
            kickback: 0.1,
            stability: 0.5,
            ..WeaponStats::default()
        }
    }

    /// Slow decay so multi-shot tests approximate "add per volley only".
    fn test_stats_slow_decay() -> WeaponStats {
        WeaponStats {
            fire_rate: 10.0,
            kickback: 0.1,
            stability: 1000.0,
            ..WeaponStats::default()
        }
    }

    #[test]
    fn kickback_rises_per_volley_without_cap() {
        let mut w = Weapon::new(test_stats_slow_decay());
        assert_eq!(w.kickback, 0.0);
        assert_eq!(w.tick(0.0, true), 1);
        assert!((w.kickback - 0.1).abs() < 1e-5);
        assert_eq!(w.tick(0.1, true), 1);
        assert!((w.kickback - 0.2).abs() < 1e-4);
        for _ in 0..10 {
            let _ = w.tick(0.1, true);
        }
        // Previously capped at 0.5 rad with `kickback_max`.
        assert!(w.kickback > 0.5 + 1e-3, "{}", w.kickback);
    }

    #[test]
    fn kickback_decays_when_not_firing() {
        let mut w = Weapon::new(test_stats());
        let _ = w.tick(0.0, true);
        assert!(w.kickback > 0.0);
        let v0 = w.kickback;
        let _ = w.tick(0.1, false);
        assert!(w.kickback < v0);
        for _ in 0..80 {
            let _ = w.tick(0.05, false);
        }
        assert!(w.kickback < 1e-4, "{}", w.kickback);
    }

    #[test]
    fn kickback_decays_while_fire_held_between_volleys() {
        let mut w = Weapon::new(test_stats());
        w.kickback = 1.0;
        w.state = WeaponFiringState::Cooldown(0.5);
        let v0 = w.kickback;
        assert_eq!(w.tick(0.1, true), 0);
        assert!(w.kickback < v0);
    }
}
