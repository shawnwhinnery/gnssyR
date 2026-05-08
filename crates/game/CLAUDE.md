# CLAUDE.md — game

## Scope

`game` contains gameplay-facing logic and scenes built on top of `gfx`, `input`, and `window`. The gameplay spec is still evolving, but rendering/test infrastructure already exists.

## Source of Truth

- Current spec placeholder: `crates/game/game.md`
- Collision presets: `src/physics_layers.rs` (pairs with `crates/physics/index.md` for the `Body` layer/mask rules)
- Integration tests: `crates/game/tests/integration/main.rs`
- Production scenes: `crates/game/src/scenes/` — `Scene` trait + `SandboxScene`
- Test-only scenes: `crates/game/tests/integration/scenes/` — scenes used exclusively for snapshot tests (e.g. `GfxShowcaseScene`)
- Runtime glue: `crates/game/src/main.rs`, `crates/game/src/lib.rs`

## Scene Management

### Trait and lifecycle

```
pub trait Scene {
    fn tick(&mut self, events: &[InputEvent]) -> Option<SceneTransition>;
    fn draw(&self, driver: &mut dyn gfx::GraphicsDriver);
    fn draw_ui(&self, _ctx: &egui::Context) {}  // default no-op
}
```

Lifecycle per frame (driven by `main.rs`):
1. `tick(events)` — advance simulation; consume input; return a transition if the scene should change
2. If `Some(SceneTransition::Replace(next))` — drop current scene, swap in `next`
3. `draw(driver)` — render gfx content via `SoftwareDriver` or `WgpuDriver`
4. `draw_ui(ctx)` — render egui overlay (only called on the egui path; headless tests skip this)

### SceneTransition

- `Replace(Box<dyn Scene>)` — drop the current scene (RAII) and start the next one immediately
- `Quit` — signal application exit (TODO: not yet wired to winit exit in `main.rs`)

### Pause composition pattern

`PauseState` is a scene-agnostic component embedded in scenes that need a pause menu:

```rust
// In SandboxScene::tick:
self.pause.tick(events);          // reads Escape key-down, toggles Playing↔Paused
if !self.pause.is_paused() {
    self.world.tick(events);      // skip simulation while paused
}

// In SandboxScene::draw_ui:
self.pause.draw_ui(ctx);          // renders the egui overlay when paused
```

`PauseState.mode` is a `Cell<GameMode>` so `draw_ui` can write back via the Resume button from `&self`. Follow this pattern for any interactive overlay that mutates state from within `draw_ui`.

### Production scenes

| Scene | Path | Description |
|-------|------|-------------|
| `SandboxScene` | `src/scenes/sandbox/` | Main play scene: `World` + `PauseState`; egui **Sandbox** window (**Primary weapon** tab edits `WeaponStats` + `ProjectileBehavior` live via `weapon_editor`) |

New production scenes go under `src/scenes/<name>/`.

## Current Project Guarantees

- Supports headless/integration testing using `SoftwareDriver` + `SimulatedBackend`.
- Snapshot regression test protects visual output of the GFX showcase scene.
- Golden snapshot file lives at `crates/game/tests/snapshots/gfx_scene.bin`.
- Set `UPDATE_SNAPSHOTS=1` when intentionally updating scene visuals.

## UI (egui)

Scenes opt into egui by overriding `Scene::draw_ui(&self, ctx: &egui::Context)` (default no-op). The method is called once per frame from the render closure in `main.rs`, inside an active egui pass.

- Use `egui::Window` / `egui::Area` for floating panels; use `egui::CentralPanel` for full-screen overlays
- `PauseState::draw_ui` is the reference implementation — shows a centered modal with a Resume button
- `PauseState.mode` uses `Cell<GameMode>` so UI callbacks can toggle state from `&self`; follow this pattern for other interactive overlays that need to write back to `&self` fields

## Editing Guidance

- Keep game-side code compatible with both headless tests and interactive runtime.
- Production scenes (`src/scenes/`) are used by both the runtime and integration tests. Test-only scenes (`tests/integration/scenes/`) are used exclusively by snapshot tests — do not import them from the library crate.
- Treat snapshot diffs as signal: confirm intentional visual changes before updating goldens.
- As gameplay spec solidifies, promote requirements from `game.md` into concrete tests first.

## Primary weapon (`weapon.rs`)

`Weapon` owns `stats: WeaponStats`, `projectile_behavior: ProjectileBehavior`, firing state, and runtime `kickback`. Each spawned shot snapshots `WeaponStats` and motion at fire time.

### `ProjectileBehavior` (movement kind)

| Variant | Implementation |
|---------|----------------|
| `Physics` | `PhysicsWorld` rigid body (bouncy, friction + min-speed cull, optional max wall bounces). `piercing` does **not** despawn physics shots. |
| `Bullet` | Kinematic straight line; circle overlap (`narrow::detect`) vs walls / targets. |
| `Rocket` | Kinematic + `rocket_acceleration`; impact damage `damage_total + kinetic_damage_scale * speed`. |
| `Oscillating` | Kinematic: forward drift + lateral sine (`oscillation_frequency`, `oscillation_magnitude`). |
| `Seeking` | Kinematic in **aim direction** at spawn; `seek_target: Option<BodyHandle>`; steers toward target with max turn rate `speed / seeking_turn_radius` (rad/s); re-acquires when the target body is removed (`PhysicsWorld::try_body`). Prefers targets inside a forward cone (`seeking_acquire_half_angle`). |

Enemies use `Enemy::projectile_behavior()` (default `Physics`); `Dummy` uses `self.weapon.projectile_behavior`.

### `WeaponStats` (edited in sandbox; copied onto each `Projectile`)

| Field | Role |
|-------|------|
| `fire_rate`, `burst_count`, `burst_delay` | Full-auto cadence and burst pacing |
| `projectiles_per_shot`, `shot_arc` | Multi-projectile spread pattern |
| `jitter` | Per-projectile random angular error (half-width, radians) |
| `kickback` | Per-volley add to runtime `Weapon::kickback` (radians); no cap — accumulator grows unbounded |
| `sway` | Seconds τ: each tick `Weapon::kickback *= exp(-dt / τ)` while `kickback > 0` (including while firing) |
| `projectile_speed`, `projectile_size`, `projectile_lifetime` | Shared projectile tuning |
| `piercing` | **Non-physics only**: extra **actor** hits before despawn (player shots → enemies; enemy shots → players). Walls always remove scripted shots. Ignored for `Physics` projectiles. |
| `damage_total`, `recoil_force` | Hit damage and owner impulse |
| `oscillation_frequency`, `oscillation_magnitude` | Oscillating path |
| `physics_max_bounces`, `physics_friction`, `physics_min_speed` | Physics projectile lifetime / damping |
| `rocket_acceleration`, `kinetic_damage_scale` | Rocket motion + damage |
| `seeking_turn_radius`, `seeking_acquire_half_angle` | Seeking turn cap + acquisition cone |

`Weapon::tick(dt, fire_intent)` advances the firing state machine and updates `kickback`. `World` spawns volleys when `tick` returns `> 0`, using `volley_directions(facing)` (spread uses `jitter + kickback`).

## Collision layers (`physics_layers.rs`)

`physics::Body` carries `collision_layers` and `collision_mask` (`u32` bitmasks). `PhysicsWorld::step` only runs broadphase/narrowphase/resolution when `Body::collides_with` is true for the pair.

- **Presets** live in `physics_layers.rs`: `wall_collision`, `player_collision`, `enemy_collision`, `npc_collision`, `projectile_player_owned`, `projectile_enemy_owned`. New walls, players, enemies, or NPCs should use the matching helper when constructing a `Body` (see `dummy.rs`, `player.rs`, `forgemaster.rs`, `world.rs` spawn, sandbox `add_sandbox_walls`).
- **Projectiles** use separate layers for player-owned vs enemy-owned shots so dense volleys do not interact with each other and shots do not physically hit their owner type.
- **Tests / one-off bodies** in other crates can set both fields to `physics::COLLISION_FILTER_MATCH_ALL` (`!0`) to preserve “collide with everything” behaviour.

## Validation

- Run game tests: `cargo test -p game`
- For intentional visual updates: `UPDATE_SNAPSHOTS=1 cargo test -p game`
