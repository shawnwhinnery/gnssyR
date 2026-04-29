use glam::Vec2;
use physics::BodyHandle;
use rand::Rng as _;

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
    /// Muzzle velocity of each projectile (world units per second).
    pub projectile_speed: f32,
    /// Collision radius of each projectile.
    pub projectile_size: f32,
    /// Maximum time in seconds a projectile lives before despawning.
    pub projectile_lifetime: f32,
    /// Number of wall hits before a projectile despawns (0 = despawn on first hit).
    pub piercing: u32,
    /// Flat base damage per projectile hit.
    pub damage_total: f32,
    /// Backward impulse applied to the owner body when a shot is fired.
    pub recoil_force: f32,
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
            projectile_speed: 15.0,
            projectile_size: 0.08,
            projectile_lifetime: 2.0,
            piercing: 0,
            damage_total: 10.0,
            recoil_force: 0.5,
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
        Self { stats, state: WeaponFiringState::Idle }
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
                        self.state =
                            WeaponFiringState::Cooldown(1.0 / self.stats.fire_rate);
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

            WeaponFiringState::Burst { ref mut remaining, ref mut next_time } => {
                *next_time -= dt;
                if *next_time <= 0.0 {
                    shots = 1;
                    *remaining -= 1;
                    if *remaining == 0 {
                        self.state =
                            WeaponFiringState::Cooldown(1.0 / self.stats.fire_rate);
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

        let mut jitter = || -> f32 {
            if self.stats.jitter > 0.0 {
                rng.gen_range(-self.stats.jitter..self.stats.jitter)
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

pub struct Projectile {
    pub body: BodyHandle,
    /// Slot index of the player who fired this projectile (used for collision filtering).
    pub owner_slot: usize,
    /// Remaining seconds before the projectile despawns due to age.
    pub lifetime: f32,
    /// Remaining wall hits before despawn (0 = despawn on first hit).
    pub piercing: u32,
    /// Visual/collision radius (world units), mirroring the physics body.
    pub size: f32,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Rotate a 2-D vector by `angle` radians.
fn rotate(v: Vec2, angle: f32) -> Vec2 {
    let (sin, cos) = angle.sin_cos();
    Vec2::new(v.x * cos - v.y * sin, v.x * sin + v.y * cos)
}
