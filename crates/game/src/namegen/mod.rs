mod gun_words;
mod mod_words;

use rand::{seq::SliceRandom as _, Rng};

use crate::weapon::WeaponStats;
use gun_words::*;
use mod_words::*;

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

pub fn gun_name(stats: &WeaponStats, rng: &mut impl Rng) -> String {
    let archetype = pick_archetype(stats, rng);
    let adj = pick_gun_adjective(stats, rng);
    let suffix = if rng.gen_bool(0.2) { Some(pick(SUFFIXES, rng)) } else { None };

    match (adj, suffix) {
        (Some(a), Some(s)) => format!("{a} {archetype} {s}"),
        (Some(a), None) => format!("{a} {archetype}"),
        (None, Some(s)) => format!("{archetype} {s}"),
        (None, None) => archetype.to_string(),
    }
}

pub fn mod_name(rng: &mut impl Rng) -> String {
    let adj = pick(MOD_ADJECTIVES, rng);
    let noun = pick(MOD_NOUNS, rng);
    format!("{adj} {noun}")
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn pick(list: &[&'static str], rng: &mut impl Rng) -> &'static str {
    list.choose(rng).copied().unwrap_or("Unknown")
}

/// Selects the archetype noun based on the weapon's most dominant mechanical trait.
/// Priority is ordered from most-distinctive to least.
fn pick_archetype(stats: &WeaponStats, rng: &mut impl Rng) -> &'static str {
    if stats.piercing >= 2 {
        pick(ARCHETYPES_PIERCE, rng)
    } else if stats.projectiles_per_shot >= 4 {
        pick(ARCHETYPES_SCATTER, rng)
    } else if stats.burst_count >= 3 {
        pick(ARCHETYPES_BURST, rng)
    } else if stats.fire_rate >= 6.0 {
        pick(ARCHETYPES_AUTO, rng)
    } else if stats.damage_total >= 20.0 {
        pick(ARCHETYPES_HEAVY, rng)
    } else {
        pick(ARCHETYPES_GENERIC, rng)
    }
}

/// Picks an adjective highlighting a *secondary* notable stat (65% chance overall).
/// Avoids re-stating what the archetype already communicates.
fn pick_gun_adjective(stats: &WeaponStats, rng: &mut impl Rng) -> Option<&'static str> {
    if !rng.gen_bool(0.65) {
        return None;
    }

    // Evaluate secondary traits in priority order.
    if stats.jitter > 0.15 {
        Some(pick(ADJ_WILD, rng))
    } else if stats.jitter < 0.03 && stats.projectiles_per_shot == 1 {
        Some(pick(ADJ_PRECISE, rng))
    } else if stats.fire_rate >= 5.0 && stats.burst_count < 3 && stats.projectiles_per_shot < 4 {
        Some(pick(ADJ_RAPID, rng))
    } else if stats.damage_total >= 18.0 && stats.fire_rate < 6.0 {
        Some(pick(ADJ_HEAVY, rng))
    } else if stats.burst_count >= 2 && stats.projectiles_per_shot < 4 {
        Some(pick(ADJ_BURST, rng))
    } else if stats.projectiles_per_shot >= 2 && stats.burst_count < 3 {
        Some(pick(ADJ_SCATTER, rng))
    } else {
        None
    }
}
