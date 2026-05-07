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
- `shot_arc`: Radians to spread projectiles across (0 = straight; up to 360° in editor).
- `burst_count`: Number of rounds fired per trigger pull.
- `burst_delay`: Time between rounds in a burst.
- `jitter`: Per-projectile random angular spread (half-width, radians).
- **`kickback`**: Each volley adds to runtime `Weapon::kickback` (radians of extra spread); **no cap**; combined with `jitter` in `volley_directions`.
- **`stability`**: Seconds τ — while the trigger is not held, `kickback` decays as `exp(-dt / τ)` (smaller τ clears bloom faster).
- `spawn_offsets`: Planned — side/forward offsets for multi-barrel or wing-mounted patterns.

### Projectile stats (`WeaponStats`, snapshot per shot)
- `projectile_speed`: Muzzle / along-path speed (world units per second); for `Rocket`, also the initial speed before acceleration.
- `projectile_size`: Collision radius.
- `projectile_lifetime`: Max duration in seconds.
- `piercing`: **Scripted** shots only — extra **actor** hits before despawn (player shots → enemies; enemy shots → players). **Walls always remove** scripted projectiles. **Ignored** for `ProjectileBehavior::Physics` (physics shots use TTL, friction/min speed, and optional max wall bounces instead).
- **Oscillating:** `oscillation_frequency` (Hz), `oscillation_magnitude` (world units lateral).
- **Physics projectile:** `physics_max_bounces` (0 = unlimited), `physics_friction` (speed damping), `physics_min_speed` (despawn threshold).
- **Rocket:** `rocket_acceleration`, `kinetic_damage_scale` (adds `kinetic_damage_scale * speed` to impact damage).
- **Seeking:** `seeking_turn_radius` (smaller = tighter turns; max turn rate ≈ `speed / radius` rad/s), `seeking_acquire_half_angle` (forward cone for picking / re-picking a `BodyHandle` target).

### Projectile behavior (`Weapon` + `ProjectileBehavior`)
`Weapon::projectile_behavior` chooses how new shots move. Implemented in `crates/game/src/world.rs` + `weapon.rs`:

| Behavior | Summary |
|----------|---------|
| `Physics` | Full rigid body in `PhysicsWorld`; bouncy; post-step friction and min-speed cull. |
| `Bullet` | Straight kinematic ray; geometric hits. |
| `Rocket` | Accelerates along heading; damage scales with speed. |
| `Oscillating` | Forward motion + perpendicular sine wave. |
| `Seeking` | Fires along **aim**; tracks `seek_target` with turn-rate limit; re-acquires if the target body is removed (`PhysicsWorld::try_body`). |

Sandbox **Primary weapon** tab: `egui::ComboBox` for behavior plus grouped rows for oscillating / physics / rocket / seeking stats (`weapon_editor` in `sandbox/mod.rs`).

### Elemental Damage Pool
- `damage_total`: Flat base damage per projectile hit (implemented).
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

### Physics integration (`world.rs` + `physics_layers.rs`)
- **Collision layers**: `Physics` projectiles get `projectile_player_owned` / `projectile_enemy_owned` masks (see `physics_layers.rs` and `crates/physics/index.md` **Body**). Scripted shots have **no** projectile `Body`; hits use `narrow::detect` against walls and actor bodies.
- **Recoil**: On each volley, `facing * -recoil_force` is added to the shooter’s body velocity.

### Tick order (high level, `World::tick`)
1. Weapons / AI produce spawn batches (`WeaponStats` + `ProjectileBehavior` snapshot at spawn).
2. `PhysicsWorld::step` — moves rigid bodies (including `Physics` projectiles).
3. Physics-projectile friction, wall-bounce counting; **then** `integrate_scripted_projectiles` (Bullet / Rocket / Oscillating / Seeking).
4. Lifetime decay; `resolve_damage` (contacts for physics, circle tests for scripted).
5. `cleanup_dead_enemies`; then **`cleanup_projectiles` uses a fresh enemy handle list** so dead enemies never leave stale `BodyHandle`s in overlap paths.

## Sandbox UI

The **Primary weapon** tab exposes every `WeaponStats` field (labels match struct names), **`projectile_behavior`** (`ComboBox`), grouped sections (firing, shot pattern, spread & kickback, projectile, oscillating, physics projectile, rocket, seeking, impact), and read-only **kickback (live)**.

## Phase 1 Implementation Tasks
1. **Scaffold `weapon.rs`**: Define `WeaponStats`, `Weapon`, and `Projectile` structs. *(Done — extended with kickback / stability as above.)*
2. **Integrate `Weapon` into `Player`**: Update `player.rs` to include a `Weapon` field.
3. **Basic Spawning**: In `world.rs`, handle the `fire` intent by calling a new `spawn_projectile` function.
4. **Projectile Tick**: Add `tick_projectiles` to `world.rs` to move projectiles and decrement lifetime.
5. **Collision Detection**: Use `physics.contacts()` to detect when projectiles hit walls and remove them.

