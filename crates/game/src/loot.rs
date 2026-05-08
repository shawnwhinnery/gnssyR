use glam::Vec2;
use rand::Rng;

use crate::{
    namegen,
    weapon::{Weapon, WeaponStats},
};

// ---------------------------------------------------------------------------
// Stat distribution primitive
// ---------------------------------------------------------------------------

/// A continuous stat range sampled with a quadratic-curve skew.
///
/// `min` and `max` use natural numeric ordering (min ≤ max).
/// `lower_is_better` flips the skew direction so rare rolls land near `min`.
/// `skew` ∈ [-1, 1]: -1 weights toward `min`, 0 is uniform, 1 weights toward `max`.
pub struct StatRange {
    pub min: f32,
    pub max: f32,
    pub skew: f32,
    pub lower_is_better: bool,
}

impl StatRange {
    pub fn roll(&self, rng: &mut impl Rng) -> f32 {
        let t: f32 = rng.gen();
        // Quadratic blend: t + s*t*(1-t) where s ∈ [-1,1].
        // s=-1 → t² (ease-in, weights toward 0/min)
        // s= 0 → uniform
        // s= 1 → 2t-t² (ease-out, weights toward 1/max)
        let s = self.skew.clamp(-1.0, 1.0);
        let t_curved = (t + s * t * (1.0 - t)).clamp(0.0, 1.0);
        let t_final = if self.lower_is_better { 1.0 - t_curved } else { t_curved };
        self.min + (self.max - self.min) * t_final
    }
}

/// Weighted tier roll for discrete stats.
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

// ---------------------------------------------------------------------------
// Per-stat drop ranges
//
// Skew math reference (skew k, range [min, max]):
//   Median t = 0.5^k  →  median output depends on lower_is_better direction
//   Expected t = 1/(k+1)
//
// All ranges use natural ordering (min ≤ max).
// lower_is_better=true: rare rolls approach min; common rolls approach max.
// lower_is_better=false: rare rolls approach max; common rolls approach min.
// ---------------------------------------------------------------------------
const BASE: WeaponStats = WeaponStats::defaults();
const SKEW: f32 = 0.0;
const FIRE_RATE: StatRange = StatRange {
    min: BASE.fire_rate,
    max: BASE.fire_rate * 5.0,
    skew: SKEW,
    lower_is_better: false,
};
const SWAY: StatRange = StatRange {
    min: 0.0,
    max: BASE.sway,
    skew: SKEW,
    lower_is_better: true,
};
const SHOT_ARC: StatRange = StatRange {
    min: BASE.shot_arc * 0.5,
    max: BASE.shot_arc,
    skew: SKEW,
    lower_is_better: true,
};
const BURST_DELAY: StatRange = StatRange {
    min: BASE.burst_delay * 0.5,
    max: BASE.burst_delay,
    skew: SKEW,
    lower_is_better: true,
};
const JITTER: StatRange = StatRange {
    min: 0.0,
    max: BASE.jitter,
    skew: SKEW,
    lower_is_better: true,
};
const PROJ_SPEED: StatRange = StatRange {
    min: BASE.projectile_speed,
    max: BASE.projectile_speed * 1.5,
    skew: SKEW,
    lower_is_better: false,
};
const PROJ_SIZE: StatRange = StatRange {
    min: BASE.projectile_size,
    max: BASE.projectile_size * 3.0,
    skew: SKEW,
    lower_is_better: false,
};
const PROJ_LIFETIME: StatRange = StatRange {
    min: BASE.projectile_lifetime,
    max: BASE.projectile_lifetime * 1.1,
    skew: SKEW,
    lower_is_better: false,
};
const DAMAGE_TOTAL: StatRange = StatRange {
    min: BASE.damage_total,
    max: BASE.damage_total * 3.0,
    skew: SKEW,
    lower_is_better: false,
};
const RECOIL_FORCE: StatRange = StatRange {
    min: 0.0,
    max: BASE.damage_total,
    skew: SKEW,
    lower_is_better: true,
};
const KICKBACK: StatRange = StatRange {
    min: 0.0,
    max: BASE.kickback,
    skew: SKEW,
    lower_is_better: true,
};
const OSCILLATION_FREQUENCY: StatRange = StatRange {
    min: BASE.oscillation_frequency,
    max: BASE.oscillation_frequency * 0.5,
    skew: SKEW,
    lower_is_better: false,
};
const OSCILLATION_MAGNITUDE: StatRange = StatRange {
    min: BASE.oscillation_magnitude,
    max: BASE.oscillation_magnitude * 3.0,
    skew: SKEW,
    lower_is_better: false,
};
const PHYSICS_FRICTION: StatRange = StatRange {
    min: BASE.physics_friction,
    max: 0.0,
    skew: SKEW,
    lower_is_better: true,
};
const PHYSICS_MIN_SPEED: StatRange = StatRange {
    min: 0.0,
    max: BASE.physics_min_speed * 2.0,
    skew: SKEW,
    lower_is_better: true,
};
const ROCKET_ACCELERATION: StatRange = StatRange {
    min: BASE.rocket_acceleration,
    max: BASE.rocket_acceleration * 3.0,
    skew: SKEW,
    lower_is_better: false,
};
const KINETIC_DAMAGE_SCALE: StatRange = StatRange {
    min: BASE.kinetic_damage_scale,
    max: BASE.kinetic_damage_scale * 3.0,
    skew: SKEW,
    lower_is_better: false,
};
const SEEKING_TURN_RADIUS: StatRange = StatRange {
    min: BASE.seeking_turn_radius * 0.25,
    max: BASE.seeking_turn_radius,
    skew: SKEW,
    lower_is_better: true,
};
const SEEKING_ACQUIRE_HALF_ANGLE: StatRange = StatRange {
    min: BASE.seeking_acquire_half_angle * 0.5,
    max: std::f32::consts::PI,
    skew: SKEW,
    lower_is_better: false,
};

// Tier tables: (value, weight). Weights sum to 1.0.
const TIERS_PROJ_PER_SHOT: &[(u32, f32)] = &[(0, 0.80), (1, 0.10), (2, 0.05), (3, 0.03), (4, 0.02)];

const TIERS_BURST_COUNT: &[(u32, f32)] = &[(1, 0.80), (3, 0.10), (5, 0.05), (8, 0.03), (10, 0.02)];

const TIERS_PIERCING: &[(u32, f32)] = &[(0, 0.80), (1, 0.10), (2, 0.05), (3, 0.03), (4, 0.02)];

const TIERS_PHYSICS_MAX_BOUNCES: &[(u32, f32)] = &[(0, 0.50), (1, 0.20), (2, 0.15), (3, 0.10), (4, 0.05)];

// ---------------------------------------------------------------------------
// Base weapon template
// ---------------------------------------------------------------------------

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
/// 1.0 = best possible roll, 0.0 = worst (or not rolled by the loot system).
/// Direction is handled by each range's `lower_is_better` flag, so callers
/// always get higher = rarer/better regardless of whether the stat itself
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
            fire_rate: range_frac(stats.fire_rate, &FIRE_RATE),
            projectiles_per_shot: tier_frac(stats.projectiles_per_shot, TIERS_PROJ_PER_SHOT),
            shot_arc: range_frac(stats.shot_arc, &SHOT_ARC),
            burst_count: tier_frac(stats.burst_count, TIERS_BURST_COUNT),
            burst_delay: range_frac(stats.burst_delay, &BURST_DELAY),
            jitter: range_frac(stats.jitter, &JITTER),
            kickback: range_frac(stats.kickback, &KICKBACK),
            sway: range_frac(stats.sway, &SWAY),
            projectile_speed: range_frac(stats.projectile_speed, &PROJ_SPEED),
            projectile_size: range_frac(stats.projectile_size, &PROJ_SIZE),
            projectile_lifetime: range_frac(stats.projectile_lifetime, &PROJ_LIFETIME),
            piercing: tier_frac(stats.piercing, TIERS_PIERCING),
            damage_total: range_frac(stats.damage_total, &DAMAGE_TOTAL),
            recoil_force: range_frac(stats.recoil_force, &RECOIL_FORCE),
            oscillation_frequency: range_frac(stats.oscillation_frequency, &OSCILLATION_FREQUENCY),
            oscillation_magnitude: range_frac(stats.oscillation_magnitude, &OSCILLATION_MAGNITUDE),
            physics_max_bounces: tier_frac(stats.physics_max_bounces, TIERS_PHYSICS_MAX_BOUNCES),
            physics_friction: range_frac(stats.physics_friction, &PHYSICS_FRICTION),
            physics_min_speed: range_frac(stats.physics_min_speed, &PHYSICS_MIN_SPEED),
            rocket_acceleration: range_frac(stats.rocket_acceleration, &ROCKET_ACCELERATION),
            kinetic_damage_scale: range_frac(stats.kinetic_damage_scale, &KINETIC_DAMAGE_SCALE),
            seeking_turn_radius: range_frac(stats.seeking_turn_radius, &SEEKING_TURN_RADIUS),
            seeking_acquire_half_angle: range_frac(stats.seeking_acquire_half_angle, &SEEKING_ACQUIRE_HALF_ANGLE),
        }
    }
}

/// Recover the rarity fraction `[0, 1]` from an absolute stat value.
///
/// Returns `0.0` when the span is zero or `value` falls outside the range.
/// Result is 1.0 for the best possible value, 0.0 for the worst.
fn range_frac(value: f32, range: &StatRange) -> f32 {
    let span = range.max - range.min;
    if span.abs() < f32::EPSILON {
        return 0.0;
    }
    let t = (value - range.min) / span;
    if !(0.0..=1.0).contains(&t) {
        return 0.0;
    }
    if range.lower_is_better { 1.0 - t } else { t }
}

/// Normalise a discrete tier extra (0..=max_tier) to `[0, 1]`.
fn tier_frac(extra: u32, tiers: &[(u32, f32)]) -> f32 {
    let max_val = tiers.iter().map(|&(v, _)| v).max().unwrap_or(0);
    if max_val == 0 {
        return 0.0;
    }
    (extra as f32 / max_val as f32).clamp(0.0, 1.0)
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
