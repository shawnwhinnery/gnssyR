# Spec: Game Loop

## Status: Placeholder

This spec will be written once the supporting infrastructure (graphics, input, window)
is implemented and the game mechanics are decided.

---

## Open Questions

- What is the core game concept? (IO-game style — needs more definition)
- Fixed timestep or variable? What target tick rate?
- How is game state structured? (entities, components, or flat structs?)
- Multiplayer: local only, or networked later?
- What constitutes a "round" / win condition?

---

## Known Requirements (from planning.md)

- Up to 4 local players (couch co-op)
- 2D vector graphics, web IO game aesthetic
- Game loop must be testable headlessly (SoftwareDriver + SimulatedBackend)
- Mechanics sandbox must be in place before the game loop is locked in

---

## Implemented today (partial)

- **`SandboxScene`** (`src/scenes/sandbox/`): floating **Sandbox** panel with a **Primary weapon** tab — edits `WeaponStats` and **`ProjectileBehavior`** (combo + grouped stat rows), plus read-only **kickback (live)**; other tabs for enemies / inventory / forge as wired.
- **`weapon.rs`**: `WeaponStats`, `ProjectileBehavior`, `Weapon` (firing state + runtime `kickback` + `projectile_behavior`), `Projectile` / `ProjectileMotion` (physics body vs kinematic scripted); each shot snapshots stats at spawn.
- **`world.rs`**: Spawns by behavior (`Physics` vs scripted), `integrate_scripted_projectiles`, `resolve_damage`, `cleanup_projectiles`; rebuilds live enemy body lists after `cleanup_dead_enemies` so removed `BodyHandle`s are never passed to `body()`; uses `try_body` for overlap checks where handles may be stale.
- **`physics_layers.rs`**: Bitmasks and helpers (`wall_collision`, `player_collision`, `enemy_collision`, `projectile_player_owned`, …) passed into `physics::Body::collision_layers` / `collision_mask` so walls, actors, and projectiles only interact with intended categories (e.g. projectiles do not collide with each other). Spec for the filter predicate lives in `crates/physics/index.md` under **Body**.
