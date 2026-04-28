# Parameterized Weapon System Implementation Plan

## Overview
A highly modular and parameterized weapon system designed for extreme customization. Behavior emerges from numerical parameters rather than hardcoded types.

## 1. The Stat Stack (Modularity)
To support future inventory and parts, stats are separated into three layers:
- **`BaseStats`**: The "DNA" or factory settings of the weapon drop.
- **`Modifier`**: Additive or multiplicative changes (e.g., +2 projectiles, 1.2x fire rate).
- **`ComputedStats`**: The final values used for logic, recalculated whenever modifiers change.

## 2. Parameterized Behavior (The DNA)

### Shot Patterns
Defines *how* projectiles are spawned when the trigger is pulled:
- `fire_rate`: Rounds per second.
- `projectiles_per_shot`: Number of projectiles spawned simultaneously.
- `shot_arc`: Degrees/radians to spread projectiles across (0 = straight, 360 = circle).
- `burst_count`: Number of rounds fired per trigger pull.
- `burst_delay`: Time between rounds in a burst.
- `jitter`: Random variance in projectile direction/speed.
- `spawn_offsets`: Side/forward offsets for multi-barrel or wing-mounted patterns.

### Projectile Stats
Defines the physical properties of the projectile:
- `speed`: Muzzle velocity.
- `acceleration`: Change in speed over time (e.g., for rockets).
- `size`: Collision radius.
- `lifetime`: Max duration in seconds.
- `piercing`: Number of hits before despawn.

### Elemental Damage Pool
- `damage_total`: Flat base damage value.
- `ratios`: An array of 8 weights corresponding to the `Element` enum.
- `affinity`: A specific modifier that re-allocates ratios without changing `damage_total`.

```rust
pub enum Element {
    Physical, Fire, Holy, Lightning, Frost, Poison, Shadow, Arcane
}
```

## 3. Projectile "Brains" (Traits)
Projectiles carry a list of behavioral traits processed during the tick:
- **Bouncing(max_bounces)**: Reflects velocity on wall impact; decrements counter.
- **Homing(turn_rate, detection_radius)**: Steers toward the nearest enemy.
- **Splitting(child_stats_multiplier, count)**: Spawns smaller projectiles.
- **Exploding(radius, falloff)**: AoE damage on impact.
- **Accelerating(accel_rate, max_speed)**: Increases velocity magnitude over time.

## 4. Status Effects
Elemental damage has a chance to apply a status effect to the target:
- **Burn (Fire)**: Damage over time (DoT).
- **Slow (Frost)**: Multiplier to target's `move_speed`.
- **Shock (Lightning)**: Short stun; chain damage to nearby enemies.
- **Corrode (Poison)**: Increases damage taken from all sources.
- **Enfeeble (Shadow)**: Decreases damage dealt by the target.

## 5. The DNA System (Generation & Scaling)
To support the "Pokemon GO / Chao Garden" style of progression, every weapon has a hidden "DNA" that determines its potential:

### Weapon DNA
- **IVs (Individual Values)**: A set of hidden random values (0.0 to 1.0) for each `Element` and core stat (Speed, Fire Rate, etc.).
- **Scaling Multipliers**: Randomly assigned at drop; these define how much a stat grows as the weapon levels up.
- **Slot Count**: Determined by the average of all IVs (Higher average = more modifier slots).

### Leveling Logic
When a weapon levels up:
`ComputedStat = (BaseValue + (Level * ScalingFactor * IV)) * Modifiers`
This ensures a "high-IV" weapon always has a higher ceiling than a "low-IV" one, even at the same level.

## 5. The Modifier System
Modifiers are stored in a list on the weapon and applied in a specific order:
1. **Base**: Start with `BaseStats`.
2. **Add**: Apply all `Additive` modifiers (e.g., +2 projectiles).
3. **Mult**: Apply all `Multiplicative` modifiers (e.g., x1.2 damage).
4. **Override**: Boolean toggles that force a behavior (e.g., `is_full_auto = true`).
5. **Clamp**: Ensure values like `fire_rate` or `spread` don't go negative or break the engine.

### Common Modifiers
- **Barrel**: Multiplier to `projectile_speed`, additive to `spread`.
- **Trigger**: Sets `is_full_auto`, multiplier to `fire_rate`.
- **Magazine**: Additive to `mag_size`, multiplier to `reload_speed`.
- **Elemental Gem**: Re-allocates 10% of Physical damage into a specific `Element`.

## 6. Weapon State Machine
The `Weapon` struct tracks internal state to handle complex firing patterns:
- `Idle`: Ready to fire.
- `Cooldown(time)`: Waiting for next `fire_rate` interval.
- `Burst(remaining_shots, next_burst_time)`: In the middle of a burst sequence.
- `Reloading(time)`: Waiting for `reload_speed`.

## 8. Implementation Details

### Physics Integration
- **Collision Filtering**: Projectiles use a specific collision group to avoid colliding with the owner or other friendly projectiles.
- **Recoil**: When a projectile is spawned, a force vector `facing * -recoil_force` is applied to the player's `Body`.

### Tick Lifecycle
1. **`tick_weapons`**: 
    - If `reloading`, decrement timer. 
    - If `Burst`, check `next_burst_time` and spawn.
    - If `Idle` and `fire_intent`, check ammo and transition to `Burst` or `Cooldown`.
2. **`spawn_projectiles`**: 
    - Iterates `projectiles_per_shot`.
    - Calculates direction using `facing + arc_offset + jitter`.
    - Adds `Body` to `PhysicsWorld` with `Projectile` metadata in the arena.
3. **`tick_projectiles`**: 
    - Applies `Homing` or `Acceleration` forces to the physics bodies.
    - Checks `physics.contacts()` for impacts.
    - If impact: handle `Bouncing`, `Piercing`, or `Exploding`.
4. **`cleanup_projectiles`**: 
    - Removes projectiles with `lifetime <= 0` or `piercing <= 0`.
    - Synchronizes removal with `PhysicsWorld`.

## Phase 1 Implementation Tasks
1. **Scaffold `weapon.rs`**: Define `WeaponStats`, `Weapon`, and `Projectile` structs.
2. **Integrate `Weapon` into `Player`**: Update `player.rs` to include a `Weapon` field.
3. **Basic Spawning**: In `world.rs`, handle the `fire` intent by calling a new `spawn_projectile` function.
4. **Projectile Tick**: Add `tick_projectiles` to `world.rs` to move projectiles and decrement lifetime.
5. **Collision Detection**: Use `physics.contacts()` to detect when projectiles hit walls and remove them.

