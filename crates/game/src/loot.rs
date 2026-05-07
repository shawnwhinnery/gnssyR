use glam::Vec2;
use rand::Rng;

use crate::{
    namegen,
    weapon::{Weapon, WeaponStats},
};

// ---------------------------------------------------------------------------
// Stat distribution primitive
// ---------------------------------------------------------------------------

/// A continuous stat range sampled with a power-curve skew.
///
/// `skew > 1.0` concentrates results toward `min` (rarer high values).
/// `skew = 1.0` is a flat uniform distribution.
pub struct StatRange {
    pub min: f32,
    pub max: f32,
    pub skew: f32,
}

impl StatRange {
    pub fn roll(&self, rng: &mut impl Rng) -> f32 {
        let t: f32 = rng.gen::<f32>().powf(self.skew);
        self.min + (self.max - self.min) * t
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
//   Median output = min + (max - min) * 0.5^k
//   Expected output = min + (max - min) * 1/(k+1)
//
// Player defaults for reference:
//   fire_rate 5.0 | damage 10.0 | speed 15.0 | jitter 0.0
// ---------------------------------------------------------------------------

const FIRE_RATE:     StatRange = StatRange { min: 2.0, max: 10.0, skew: 1.5 };
// skew 1.5 → median ≈ 4.8, matching the player default

const SHOT_ARC:      StatRange = StatRange { min: 0.0, max: 1.4,  skew: 1.5 };
const BURST_DELAY:   StatRange = StatRange { min: 0.03, max: 0.15, skew: 1.0 };
const JITTER:        StatRange = StatRange { min: 0.0, max: 0.25, skew: 1.5 };
const PROJ_SPEED:    StatRange = StatRange { min: 10.0, max: 22.0, skew: 1.5 };
// skew 1.5 → median ≈ 14.2, close to player default 15.0

const PROJ_SIZE:     StatRange = StatRange { min: 0.06, max: 0.14, skew: 1.0 };
const PROJ_LIFETIME: StatRange = StatRange { min: 1.0, max: 4.0,  skew: 1.0 };
const DAMAGE_TOTAL:  StatRange = StatRange { min: 4.0, max: 30.0, skew: 2.0 };
// skew 2.0 → median ≈ 10.5, player default 10.0

const RECOIL_FORCE:  StatRange = StatRange { min: 0.0, max: 1.5,  skew: 1.0 };

// Tier tables: (value, weight). Weights sum to 1.0.
const TIERS_PROJ_PER_SHOT: &[(u32, f32)] =
    &[(1, 0.60), (2, 0.25), (3, 0.10), (4, 0.04), (6, 0.01)];

const TIERS_BURST_COUNT: &[(u32, f32)] =
    &[(1, 0.70), (2, 0.20), (3, 0.08), (4, 0.02)];

const TIERS_PIERCING: &[(u32, f32)] =
    &[(0, 0.80), (1, 0.15), (2, 0.04), (3, 0.01)];

// ---------------------------------------------------------------------------
// Drop generation
// ---------------------------------------------------------------------------

pub fn random_weapon_stats(rng: &mut impl Rng) -> WeaponStats {
    let projectiles_per_shot = roll_tier(TIERS_PROJ_PER_SHOT, rng);

    // Arc only matters when there is more than one projectile.
    let shot_arc = if projectiles_per_shot > 1 { SHOT_ARC.roll(rng) } else { 0.0 };

    WeaponStats {
        fire_rate: FIRE_RATE.roll(rng),
        projectiles_per_shot,
        shot_arc,
        burst_count: roll_tier(TIERS_BURST_COUNT, rng),
        burst_delay: BURST_DELAY.roll(rng),
        jitter: JITTER.roll(rng),
        projectile_speed: PROJ_SPEED.roll(rng),
        projectile_size: PROJ_SIZE.roll(rng),
        projectile_lifetime: PROJ_LIFETIME.roll(rng),
        piercing: roll_tier(TIERS_PIERCING, rng),
        damage_total: DAMAGE_TOTAL.roll(rng),
        recoil_force: RECOIL_FORCE.roll(rng),
    }
}

/// Generates a fully random weapon with a stat-derived name.
pub fn random_weapon_drop(rng: &mut impl Rng) -> (String, Weapon) {
    let stats = random_weapon_stats(rng);
    let name = namegen::gun_name(&stats, rng);
    (name, Weapon::new(stats))
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
