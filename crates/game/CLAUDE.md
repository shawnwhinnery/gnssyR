# CLAUDE.md — game

## Scope

`game` contains gameplay-facing logic and scenes built on top of `gfx`, `input`, and `window`. The gameplay spec is still evolving, but rendering/test infrastructure already exists.

## Source of Truth

- Current spec placeholder: `crates/game/game.md`
- Forgemaster / crafting system: `FORGE.md`
- Collision presets: `src/physics_layers.rs` (pairs with `crates/physics/index.md` for the `Body` layer/mask rules)
- Integration tests: `crates/game/tests/integration/main.rs`
- Production scenes: `crates/game/src/scenes/`
- Test-only scenes: `crates/game/tests/integration/scenes/` — scenes used exclusively for snapshot tests (e.g. `GfxShowcaseScene`)
- Runtime glue: `crates/game/src/main.rs`, `crates/game/src/lib.rs`

## Module Overview

```
src/
  actor.rs        — ActorCore (BodyHandle + facing), Actor trait, draw_shape helper
  camera.rs       — Camera (smooth-follow, world↔NDC, scale); HALF_VIEW = 7.28
  enemy/
    mod.rs        — Enemy trait, LootTable
    dummy.rs      — Dummy enemy (circle body, orbiting AI, fires via weapon)
  hud.rs          — HUD overlays: fps, backend name, cursor pos, collision hits
  input.rs        — InputState (per-player hold state) + InputSnapshot
  mod_part.rs     — ModPart (avg_color + blended shape), forge(), draw_mod_part()
  mode.rs         — GameMode enum (Playing / Paused)
  namegen/        — gun_name(stats), mod_name() — procedural name generation
  npc/
    mod.rs        — FriendlyNpc trait, NpcKind enum
    forgemaster.rs — Forgemaster (amber hexagon, static body, interaction_radius = 1.8)
  pause.rs        — PauseState (Cell<GameMode>, draw_ui renders egui pause overlay)
  physics_layers.rs — Collision layer bitmask helpers
  player.rs       — Player (slot, ActorCore, health, color, Weapon, weapon_name)
  scrap.rs        — Scrap, ScrapColor (8), ScrapShape (4), Inventory, draw_scrap,
                    crescent_verts / crescent_path for the crescent shape type
  weapon.rs       — WeaponStats, ProjectileBehavior, Weapon (firing state + kickback),
                    Projectile, ProjectileMotion (Physics / Scripted), ProjectileOwner
  world.rs        — World (the top-level simulation container)
  scenes/
    mod.rs        — Scene trait + SceneTransition
    main_menu/    — MainMenuScene (title screen; egui CentralPanel)
    level_select/ — LevelSelectScene (stub — "coming soon")
    level1/       — Level1Scene (two-room layout, phase state machine, door mechanic)
    sandbox/      — SandboxScene (dev sandbox; egui weapon/enemy/inventory/forge panels)
  loot.rs         — (under active refactor — do not read)
```

## Scene Management

### Trait and lifecycle

```rust
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
- `Quit` — signal application exit

### Production scenes

| Scene | Path | Description |
|-------|------|-------------|
| `MainMenuScene` | `src/scenes/main_menu/` | Title screen; "Start Game" → `Level1Scene`; "Start Sandbox" → `SandboxScene`; "Quit" → exit. Enter/Space also starts game. |
| `Level1Scene` | `src/scenes/level1/` | Two-room level with a locked door. Phase state machine: Room1 → DoorOpen (door removed) → Room2 (door re-closed) → Win / GameOver. Spawns `Dummy` enemies per phase; P1 health bar + phase banner in egui. |
| `LevelSelectScene` | `src/scenes/level_select/` | Stub placeholder ("coming soon"). Esc returns to main menu. |
| `SandboxScene` | `src/scenes/sandbox/` | Dev sandbox: `World` + `PauseState`; egui **Sandbox** window with **Primary weapon**, **Enemies**, and **Inventory** tabs. Inventory tab includes forge dialog (contribute scraps → produce `ModPart`). |

New production scenes go under `src/scenes/<name>/`.

### Pause composition pattern

`PauseState` is a scene-agnostic component embedded in scenes that need a pause menu:

```rust
// In Level1Scene::tick / SandboxScene::tick:
self.pause.tick(events);          // reads Escape key-down, toggles Playing↔Paused
if !self.pause.is_paused() {
    self.world.tick(events);      // skip simulation while paused
}

// In Level1Scene::draw_ui / SandboxScene::draw_ui:
self.pause.draw_ui(ctx, weapon_display);   // renders the egui overlay when paused
```

`PauseState.mode` is a `Cell<GameMode>` so `draw_ui` can write back via the Resume button from `&self`. Follow this pattern for any interactive overlay that mutates state from within `draw_ui`.

## World (`world.rs`)

`World` is the top-level simulation container. Fields:

| Field | Type | Role |
|-------|------|------|
| `physics` | `PhysicsWorld` | Rigid-body simulation |
| `players` | `Vec<Player>` | Up to 4 local players |
| `enemies` | `Vec<Box<dyn Enemy>>` | Active enemies |
| `npcs` | `Vec<Box<dyn FriendlyNpc>>` | Friendly NPCs (currently only `Forgemaster`) |
| `walls` | `Vec<Wall>` | Static physics bodies with a label char and fill color |
| `projectiles` | `Vec<Projectile>` | All live projectiles |
| `scraps` | `Vec<Scrap>` | Loose scrap pickups in the world |
| `weapon_drops` | `Vec<WeaponDrop>` | Dropped weapon pickups |
| `inventory` | `Inventory` | P1 scrap inventory (shared across all scenes for now) |
| `camera` | `Camera` | Smooth-follow camera tracking average live-player position |
| `time_scale` | `f32` | Multiplier on `dt`; set to 0.25 for slow-motion mode in sandbox |

Key methods:
- `World::new()` — creates an empty world with P1 at origin; callers add walls via `add_wall`
- `add_wall` / `remove_wall` — register/deregister static bodies; `remove_wall` is used by `Level1Scene` to open/close the door
- `spawn_enemy(pos)` — spawns a `Dummy` at `pos`
- `spawn_forgemaster(pos)` — spawns a `Forgemaster` NPC
- `spawn_scrap(pos, color, shape)` — directly inserts a scrap pickup
- `nearest_interactable_npc()` — returns `Option<NpcKind>` for proximity UI prompts
- `respawn_player(slot)` — resets health to 100 and position to origin

## Actor (`actor.rs`)

`ActorCore` is the shared physics + orientation state carried by players, enemies, and NPCs:

```rust
pub struct ActorCore {
    pub body: BodyHandle,
    pub facing: Vec2,          // unit vector; default Vec2::X
}
```

The `Actor` trait requires `actor() -> &ActorCore` and `draw(...)`. `draw_shape` is a free function that tessellates a `Path` and uploads/draws it in one call.

## Camera (`camera.rs`)

`Camera` converts between world space and NDC (normalised device coordinates, ±1):

- `HALF_VIEW = 7.28` world units — the half-width of the visible area
- `world_to_ndc(p)` / `ndc_to_world(p)` — coordinate conversion
- `scale(world_len)` — converts a world-space length to an NDC length for drawing
- `update(target, dt)` — smooth-follow using a critically-damped spring approximation; `smooth_time = 0.3 s` by default; `World::tick` calls this tracking the average of all live players

## Scrap & Inventory (`scrap.rs`)

Scraps are small collectible pickups dropped by enemies.

- `ScrapColor` — 8 variants: Red, Orange, Yellow, Green, Cyan, Blue, Purple, Pink
- `ScrapShape` — 4 variants: Diamond, Circle, Crescent, Triangle
- `Scrap { position, color, shape }` — in-world entity (drawn as a small coloured shape)
- `Inventory` — fixed 64-byte table (8 × 4 × u16 counts); `add`, `count`, `remove`, `count_shape`, `total`
- Collection radius: `PLAYER_RADIUS + 0.18` world units; players auto-collect on overlap
- `crescent_verts(r)` and `crescent_path(center, r)` — shared between `scrap.rs` and `mod_part.rs`

## ModPart & Forge (`mod_part.rs`)

`ModPart` is a weapon modifier produced by the Forgemaster from scrap contributions:

```rust
pub struct ModPart {
    pub avg_color: [f32; 3],  // weighted RGB average of contributing scraps
    pub shape: Vec<Vec2>,     // 12-vertex blended polygon (world-space, centered at origin)
}
```

`forge(contributions: &[(ScrapColor, ScrapShape, u16)]) -> Option<ModPart>`:
- Weighted-average RGB over all contributing scraps
- Each shape type resampled to 12 vertices, weight-averaged by contribution count
- ±10% per-vertex random noise for organic variation
- Returns `None` if total count is zero

`draw_mod_part(part, center, driver)` draws the polygon with a solid fill matching `avg_color`.

See `FORGE.md` for the full crafting system design.

## NPC System (`npc/`)

```rust
pub trait FriendlyNpc {
    fn actor(&self) -> &ActorCore;
    fn body(&self) -> BodyHandle;         // default: actor().body
    fn interaction_radius(&self) -> f32;
    fn kind(&self) -> NpcKind;
    fn draw(&self, physics, driver, camera);
}

pub enum NpcKind { Forgemaster }
```

`Forgemaster` — amber hexagon, infinite-mass static body, `interaction_radius = 1.8`. Spawned by `World::spawn_forgemaster`. The sandbox scene detects proximity via `World::nearest_interactable_npc()` and shows an `[E] Forgemaster` egui prompt; pressing E opens the forge dialog.

## Name Generation (`namegen/`)

```rust
pub fn gun_name(stats: &WeaponStats, rng: &mut impl Rng) -> String
pub fn mod_name(rng: &mut impl Rng) -> String
```

`gun_name` selects an archetype noun from the weapon's dominant trait (piercing ≥ 2, scatter ≥ 4 projectiles, burst ≥ 3, auto ≥ 6 rps, heavy ≥ 20 damage, else generic), then optionally prepends a secondary adjective and a suffix. `mod_name` combines a random adjective + noun from `mod_words`. Both use internal word-list tables in `gun_words.rs` / `mod_words.rs`.

## Loot & Weapon Drops

Enemy `LootTable` controls:
- `min_drops` / `max_drops` — scrap count range on death
- `weapon_drop_chance` — probability [0, 1] of also dropping a `WeaponDrop`

`WeaponDrop { position, name: String, weapon: Weapon }` is collected by any live player within `PLAYER_RADIUS + 0.35` world units; the collected weapon replaces the player's current weapon and sets `player.weapon_name`.

> Note: `loot.rs` (random weapon drop generation) is under active refactor — do not read or modify it without checking first.

## Primary weapon (`weapon.rs`)

`Weapon` owns `stats: WeaponStats`, `projectile_behavior: ProjectileBehavior`, firing state, and runtime `kickback`. Each spawned shot snapshots `WeaponStats` and motion at fire time.

### `ProjectileBehavior` (movement kind)

| Variant | Implementation |
|---------|----------------|
| `Bullet` | Kinematic straight line; circle overlap (`narrow::detect`) vs walls / targets. **Default.** |
| `Physics` | `PhysicsWorld` rigid body (bouncy, friction + min-speed cull, optional max wall bounces). `piercing` does **not** despawn physics shots. |
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

`WeaponStats::defaults()` is a `const fn` so `loot.rs` can reference field values in `const StatRange` definitions.

`Weapon::tick(dt, fire_intent)` advances the firing state machine and updates `kickback`. `World` spawns volleys when `tick` returns `> 0`, using `volley_directions(facing)` (spread uses `jitter + kickback`).

## Collision layers (`physics_layers.rs`)

`physics::Body` carries `collision_layers` and `collision_mask` (`u32` bitmasks). `PhysicsWorld::step` only runs broadphase/narrowphase/resolution when `Body::collides_with` is true for the pair.

- **Presets** live in `physics_layers.rs`: `wall_collision`, `player_collision`, `enemy_collision`, `npc_collision`, `projectile_player_owned`, `projectile_enemy_owned`. New walls, players, enemies, or NPCs should use the matching helper when constructing a `Body` (see `dummy.rs`, `player.rs`, `forgemaster.rs`, `world.rs` spawn, sandbox `add_sandbox_walls`).
- **Projectiles** use separate layers for player-owned vs enemy-owned shots so dense volleys do not interact with each other and shots do not physically hit their owner type.
- **Tests / one-off bodies** in other crates can set both fields to `physics::COLLISION_FILTER_MATCH_ALL` (`!0`) to preserve "collide with everything" behaviour.

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
- Deferred mutations from `draw_ui` use `Cell<bool>` flags consumed in the next `tick` call (see `enemy_spawn_requests`, `forge_requested`, `return_to_menu`, `restart` etc.)

## Editing Guidance

- Keep game-side code compatible with both headless tests and interactive runtime.
- Production scenes (`src/scenes/`) are used by both the runtime and integration tests. Test-only scenes (`tests/integration/scenes/`) are used exclusively by snapshot tests — do not import them from the library crate.
- Treat snapshot diffs as signal: confirm intentional visual changes before updating goldens.
- As gameplay spec solidifies, promote requirements from `game.md` into concrete tests first.
- Do not read or modify `loot.rs` without checking first — it is under active refactor.

## Validation

- Run game tests: `cargo test -p game`
- For intentional visual updates: `UPDATE_SNAPSHOTS=1 cargo test -p game`
