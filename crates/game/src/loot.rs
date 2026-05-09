use glam::Vec2;
use rand::Rng;

use crate::{
    namegen,
    weapon::{Weapon, WeaponStats},
};

// ---------------------------------------------------------------------------
// Stat tier primitive
// ---------------------------------------------------------------------------

/// Five discrete tiers ordered Common (index 0) → Legendary (index 4).
/// `values[0]` is the worst/common roll; `values[4]` is the best/legendary roll.
/// Direction (higher or lower = better) is encoded in value ordering — no flag needed.
pub struct StatTiers {
    pub values:  [f32; 5],
    pub weights: [f32; 5],
}

const DEFAULT_WEIGHTS: [f32; 5] = [0.698, 0.001, 0.1, 0.1, 0.1];

impl StatTiers {
    pub fn roll(&self, rng: &mut impl Rng) -> f32 {
        let ceiling: usize = self.values.len() - 1;
        let dice = 20;
        let mut roll: usize = rand::thread_rng().gen_range(0..dice);
        if(roll > ceiling) {
            roll = ceiling;
        }
        self.values[roll]
    }

    /// Returns rarity fraction in `[0, 1]` — 0.0 for Common, 1.0 for Legendary.
    /// Returns 0.0 when `value` does not exactly match any tier.
    pub fn frac(&self, value: f32) -> f32 {
        for (i, &v) in self.values.iter().enumerate() {
            if value == v {
                return i as f32 / 4.0;
            }
        }
        0.0
    }
}

/// Weighted tier roll for discrete (u32) stats.
///
/// `tiers` is a slice of `(value, weight)` pairs; weights must sum to 1.0.
fn roll_tier(tiers: &[(u32, f32)], rng: &mut impl Rng) -> u32 {
    let roll: f32 = rng.gen();
    let mut cursor = 0.0_f32;
    for &(value, weight) in tiers {
        cursor += weight;
        if roll < cursor {
            return value;
        }
    }
    tiers.last().map(|&(v, _)| v).unwrap_or(0)
}

/// Normalise a discrete tier value to `[0, 1]` by its position in the tier table.
fn tier_frac_u32(value: u32, tiers: &[(u32, f32)]) -> f32 {
    let idx = tiers.iter().position(|&(v, _)| v == value).unwrap_or(0);
    let max_idx = tiers.len().saturating_sub(1).max(1);
    idx as f32 / max_idx as f32
}

// ---------------------------------------------------------------------------
// Per-stat drop tiers
//
// values[0] = Common (worst roll), values[4] = Legendary (best roll).
// Direction is encoded in value ordering: ascending = higher is better,
// descending = lower is better.
// ---------------------------------------------------------------------------

const BASE: WeaponStats = WeaponStats::defaults();

const FIRE_RATE: StatTiers = StatTiers {
    values:  [BASE.fire_rate, BASE.fire_rate * 2.0, BASE.fire_rate * 3.0, BASE.fire_rate * 4.0, BASE.fire_rate * 5.0],
    weights: DEFAULT_WEIGHTS,
};
const SWAY: StatTiers = StatTiers {
    values:  [BASE.sway, BASE.sway * 0.75, BASE.sway * 0.5, BASE.sway * 0.25, 0.0],
    weights: DEFAULT_WEIGHTS,
};
const SHOT_ARC: StatTiers = StatTiers {
    values:  [BASE.shot_arc, BASE.shot_arc * 0.875, BASE.shot_arc * 0.75, BASE.shot_arc * 0.625, BASE.shot_arc * 0.5],
    weights: DEFAULT_WEIGHTS,
};
const BURST_DELAY: StatTiers = StatTiers {
    values:  [BASE.burst_delay, BASE.burst_delay * 0.875, BASE.burst_delay * 0.75, BASE.burst_delay * 0.625, BASE.burst_delay * 0.5],
    weights: DEFAULT_WEIGHTS,
};
// Defined independently because BASE.jitter = 0.0 makes a BASE-relative range zero-span.
const JITTER: StatTiers = StatTiers {
    values:  [0.10, 0.07, 0.04, 0.01, 0.0],
    weights: DEFAULT_WEIGHTS,
};
const KICKBACK: StatTiers = StatTiers {
    values:  [BASE.kickback, BASE.kickback * 0.75, BASE.kickback * 0.5, BASE.kickback * 0.25, 0.0],
    weights: DEFAULT_WEIGHTS,
};
const PROJ_SPEED: StatTiers = StatTiers {
    values:  [BASE.projectile_speed, BASE.projectile_speed * 1.125, BASE.projectile_speed * 1.25, BASE.projectile_speed * 1.375, BASE.projectile_speed * 1.5],
    weights: DEFAULT_WEIGHTS,
};
const PROJ_SIZE: StatTiers = StatTiers {
    values:  [BASE.projectile_size, BASE.projectile_size * 1.5, BASE.projectile_size * 2.0, BASE.projectile_size * 2.5, BASE.projectile_size * 3.0],
    weights: DEFAULT_WEIGHTS,
};
const PROJ_LIFETIME: StatTiers = StatTiers {
    values:  [BASE.projectile_lifetime, BASE.projectile_lifetime * 1.025, BASE.projectile_lifetime * 1.05, BASE.projectile_lifetime * 1.075, BASE.projectile_lifetime * 1.1],
    weights: DEFAULT_WEIGHTS,
};
const DAMAGE_TOTAL: StatTiers = StatTiers {
    values:  [BASE.damage_total, BASE.damage_total * 1.5, BASE.damage_total * 2.0, BASE.damage_total * 2.5, BASE.damage_total * 3.0],
    weights: DEFAULT_WEIGHTS,
};
const RECOIL_FORCE: StatTiers = StatTiers {
    values:  [BASE.damage_total, BASE.damage_total * 0.75, BASE.damage_total * 0.5, BASE.damage_total * 0.25, 0.0],
    weights: DEFAULT_WEIGHTS,
};
const OSCILLATION_FREQUENCY: StatTiers = StatTiers {
    values:  [BASE.oscillation_frequency, BASE.oscillation_frequency * 0.875, BASE.oscillation_frequency * 0.75, BASE.oscillation_frequency * 0.625, BASE.oscillation_frequency * 0.5],
    weights: DEFAULT_WEIGHTS,
};
const OSCILLATION_MAGNITUDE: StatTiers = StatTiers {
    values:  [BASE.oscillation_magnitude, BASE.oscillation_magnitude * 1.5, BASE.oscillation_magnitude * 2.0, BASE.oscillation_magnitude * 2.5, BASE.oscillation_magnitude * 3.0],
    weights: DEFAULT_WEIGHTS,
};
const PHYSICS_FRICTION: StatTiers = StatTiers {
    values:  [BASE.physics_friction, BASE.physics_friction * 0.75, BASE.physics_friction * 0.5, BASE.physics_friction * 0.25, 0.0],
    weights: DEFAULT_WEIGHTS,
};
const PHYSICS_MIN_SPEED: StatTiers = StatTiers {
    values:  [BASE.physics_min_speed * 2.0, BASE.physics_min_speed * 1.5, BASE.physics_min_speed, BASE.physics_min_speed * 0.5, 0.0],
    weights: DEFAULT_WEIGHTS,
};
const ROCKET_ACCELERATION: StatTiers = StatTiers {
    values:  [BASE.rocket_acceleration, BASE.rocket_acceleration * 1.5, BASE.rocket_acceleration * 2.0, BASE.rocket_acceleration * 2.5, BASE.rocket_acceleration * 3.0],
    weights: DEFAULT_WEIGHTS,
};
const KINETIC_DAMAGE_SCALE: StatTiers = StatTiers {
    values:  [BASE.kinetic_damage_scale, BASE.kinetic_damage_scale * 1.5, BASE.kinetic_damage_scale * 2.0, BASE.kinetic_damage_scale * 2.5, BASE.kinetic_damage_scale * 3.0],
    weights: DEFAULT_WEIGHTS,
};
const SEEKING_TURN_RADIUS: StatTiers = StatTiers {
    values:  [BASE.seeking_turn_radius, BASE.seeking_turn_radius * 0.8125, BASE.seeking_turn_radius * 0.625, BASE.seeking_turn_radius * 0.4375, BASE.seeking_turn_radius * 0.25],
    weights: DEFAULT_WEIGHTS,
};
const SEEKING_ACQUIRE_HALF_ANGLE: StatTiers = StatTiers {
    values:  [
        std::f32::consts::PI * 0.25,
        std::f32::consts::PI * 0.4375,
        std::f32::consts::PI * 0.625,
        std::f32::consts::PI * 0.8125,
        std::f32::consts::PI,
    ],
    weights: DEFAULT_WEIGHTS,
};

// Tier tables for discrete (u32) stats: (value, weight). Weights sum to 1.0.
const TIERS_PROJ_PER_SHOT: &[(u32, f32)] = &[(0, 0.80), (1, 0.10), (2, 0.05), (3, 0.03), (4, 0.02)];

const TIERS_BURST_COUNT: &[(u32, f32)] = &[(1, 0.80), (3, 0.10), (5, 0.05), (8, 0.03), (10, 0.02)];

const TIERS_PIERCING: &[(u32, f32)] = &[(0, 0.80), (1, 0.10), (2, 0.05), (3, 0.03), (4, 0.02)];

const TIERS_PHYSICS_MAX_BOUNCES: &[(u32, f32)] = &[(0, 0.50), (1, 0.20), (2, 0.15), (3, 0.10), (4, 0.05)];

// ---------------------------------------------------------------------------
// Drop generation
// ---------------------------------------------------------------------------

pub fn random_weapon_stats(rng: &mut impl Rng) -> WeaponStats {
    WeaponStats {
        fire_rate: FIRE_RATE.roll(rng),
        projectiles_per_shot: roll_tier(TIERS_PROJ_PER_SHOT, rng),
        shot_arc: SHOT_ARC.roll(rng),
        burst_count: roll_tier(TIERS_BURST_COUNT, rng),
        burst_delay: BURST_DELAY.roll(rng),
        jitter: JITTER.roll(rng),
        kickback: KICKBACK.roll(rng),
        sway: SWAY.roll(rng),
        projectile_speed: PROJ_SPEED.roll(rng),
        projectile_size: PROJ_SIZE.roll(rng),
        projectile_lifetime: PROJ_LIFETIME.roll(rng),
        piercing: roll_tier(TIERS_PIERCING, rng),
        damage_total: DAMAGE_TOTAL.roll(rng),
        recoil_force: RECOIL_FORCE.roll(rng),
        oscillation_frequency: OSCILLATION_FREQUENCY.roll(rng),
        oscillation_magnitude: OSCILLATION_MAGNITUDE.roll(rng),
        physics_max_bounces: roll_tier(TIERS_PHYSICS_MAX_BOUNCES, rng),
        physics_friction: PHYSICS_FRICTION.roll(rng),
        physics_min_speed: PHYSICS_MIN_SPEED.roll(rng),
        rocket_acceleration: ROCKET_ACCELERATION.roll(rng),
        kinetic_damage_scale: KINETIC_DAMAGE_SCALE.roll(rng),
        seeking_turn_radius: SEEKING_TURN_RADIUS.roll(rng),
        seeking_acquire_half_angle: SEEKING_ACQUIRE_HALF_ANGLE.roll(rng),
    }
}

/// Generates a fully random weapon with a stat-derived name.
pub fn random_weapon_drop(rng: &mut impl Rng) -> (String, Weapon) {
    let stats = random_weapon_stats(rng);
    let name = namegen::gun_name(&stats, rng);
    (name, Weapon::new(stats))
}

// ---------------------------------------------------------------------------
// Rarity fractions
// ---------------------------------------------------------------------------

/// Per-stat rarity fraction in `[0, 1]`.
///
/// 1.0 = best possible roll (Legendary), 0.0 = worst (Common).
/// Higher is always rarer/better regardless of whether the stat itself
/// favours high or low values.
pub struct WeaponStatRarities {
    pub fire_rate: f32,
    pub projectiles_per_shot: f32,
    pub shot_arc: f32,
    pub burst_count: f32,
    pub burst_delay: f32,
    pub jitter: f32,
    pub kickback: f32,
    pub sway: f32,
    pub projectile_speed: f32,
    pub projectile_size: f32,
    pub projectile_lifetime: f32,
    pub piercing: f32,
    pub damage_total: f32,
    pub recoil_force: f32,
    pub oscillation_frequency: f32,
    pub oscillation_magnitude: f32,
    pub physics_max_bounces: f32,
    pub physics_friction: f32,
    pub physics_min_speed: f32,
    pub rocket_acceleration: f32,
    pub kinetic_damage_scale: f32,
    pub seeking_turn_radius: f32,
    pub seeking_acquire_half_angle: f32,
}

impl WeaponStatRarities {
    /// Overall weapon quality score in `[0, 1]` — used to assign a rarity tier.
    ///
    /// Averages only the four stats that most define weapon power.  Averaging all
    /// 23 stats collapses to a near-constant value (CLT: σ ≈ 0.06 regardless of
    /// skew), making every weapon land in the middle tier.  With four stats the
    /// standard deviation is ~0.14, which spreads naturally across all five tiers.
    pub fn overall_score(&self) -> f32 {
        let key = [
            self.fire_rate,
            self.damage_total,
            self.projectile_speed,
            self.burst_count,
        ];
        key.iter().sum::<f32>() / key.len() as f32
    }

    pub fn from_stats(stats: &WeaponStats) -> Self {
        Self {
            fire_rate: FIRE_RATE.frac(stats.fire_rate),
            projectiles_per_shot: tier_frac_u32(stats.projectiles_per_shot, TIERS_PROJ_PER_SHOT),
            shot_arc: SHOT_ARC.frac(stats.shot_arc),
            burst_count: tier_frac_u32(stats.burst_count, TIERS_BURST_COUNT),
            burst_delay: BURST_DELAY.frac(stats.burst_delay),
            jitter: JITTER.frac(stats.jitter),
            kickback: KICKBACK.frac(stats.kickback),
            sway: SWAY.frac(stats.sway),
            projectile_speed: PROJ_SPEED.frac(stats.projectile_speed),
            projectile_size: PROJ_SIZE.frac(stats.projectile_size),
            projectile_lifetime: PROJ_LIFETIME.frac(stats.projectile_lifetime),
            piercing: tier_frac_u32(stats.piercing, TIERS_PIERCING),
            damage_total: DAMAGE_TOTAL.frac(stats.damage_total),
            recoil_force: RECOIL_FORCE.frac(stats.recoil_force),
            oscillation_frequency: OSCILLATION_FREQUENCY.frac(stats.oscillation_frequency),
            oscillation_magnitude: OSCILLATION_MAGNITUDE.frac(stats.oscillation_magnitude),
            physics_max_bounces: tier_frac_u32(stats.physics_max_bounces, TIERS_PHYSICS_MAX_BOUNCES),
            physics_friction: PHYSICS_FRICTION.frac(stats.physics_friction),
            physics_min_speed: PHYSICS_MIN_SPEED.frac(stats.physics_min_speed),
            rocket_acceleration: ROCKET_ACCELERATION.frac(stats.rocket_acceleration),
            kinetic_damage_scale: KINETIC_DAMAGE_SCALE.frac(stats.kinetic_damage_scale),
            seeking_turn_radius: SEEKING_TURN_RADIUS.frac(stats.seeking_turn_radius),
            seeking_acquire_half_angle: SEEKING_ACQUIRE_HALF_ANGLE.frac(stats.seeking_acquire_half_angle),
        }
    }
}

// ---------------------------------------------------------------------------
// Ground pickup
// ---------------------------------------------------------------------------

/// A named weapon lying on the ground, waiting to be picked up.
pub struct WeaponDrop {
    pub position: Vec2,
    pub name: String,
    pub weapon: Weapon,
}
