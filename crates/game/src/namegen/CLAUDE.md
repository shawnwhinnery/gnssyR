# CLAUDE.md — game/src/namegen/

## Purpose

Procedural name generation for weapons and mod parts. Pure logic — no side effects, no physics or rendering dependency. Entirely driven by `WeaponStats` fields and an `Rng` seed.

## Files

| File | Responsibility |
|------|---------------|
| `mod.rs` | Public API — `gun_name(stats, rng)` and `mod_name(rng)` |
| `gun_words.rs` | Word tables and helpers for gun names: archetype nouns, adjectives, suffixes |
| `mod_words.rs` | Word tables for mod part names: adjectives and nouns |

## API

```rust
pub fn gun_name(stats: &WeaponStats, rng: &mut impl Rng) -> String
pub fn mod_name(rng: &mut impl Rng) -> String
```

### `gun_name` — archetype selection

The dominant weapon trait picks the noun archetype (checked in priority order):

| Condition | Archetype |
|-----------|-----------|
| `piercing >= 2` | piercing |
| `projectiles_per_shot >= 4` | scatter |
| `burst_count >= 3` | burst |
| `fire_rate >= 6.0 rps` | auto |
| `damage_total >= 20.0` | heavy |
| else | generic |

Then optionally prepends a secondary adjective and a suffix drawn from `gun_words.rs`.

### `mod_name`

Combines a random adjective + noun from `mod_words.rs`.

## Invariants

- No mutable global state — fully deterministic for a given `rng` seed.
- All vocabulary lives in `gun_words.rs` / `mod_words.rs`; extend word lists there.
- Both functions take `&WeaponStats` / `&mut impl Rng` — no world or game state access.
