# Game Architecture Plan — Typed Arena World

## Goal

Replace the current monolithic `GameState` struct with a proper game architecture
using the **Typed Arena** pattern. Each entity category lives in its own typed
storage; systems are plain functions that operate on `World`.

This is scoped to the `game` crate. Physics, graphics, input, and window are
unchanged. The sandbox scene is the validation target throughout.

---

## Guiding Principles

- **No framework magic.** Systems are functions, not traits. Entities are structs,
  not handle IDs.
- **Consistent with physics.** `PhysicsWorld` already uses a handle-based arena
  (`BodyHandle`). Each `Body` also carries **collision layer** bitmasks; the game
  crate centralizes presets in `physics_layers.rs`. Game-side arenas follow the same idiom.
- **Test-friendly.** `World::new()` takes no graphics or window handles. Any
  tick-level test can construct a `World`, call `world.tick(events, dt)`, and
  assert on state — no GPU, no event loop.
- **Spec-first.** Before any module is implemented, its test cases are written.

---

## Module Layout

```
crates/game/src/
  lib.rs                        — module declarations; re-exports GameMode, PauseState
  main.rs                       — thin glue: constructs the first scene, runs the egui app loop
  mode.rs                       — GameMode enum (Playing / Paused); scene-agnostic
  pause.rs                      — PauseState component; Escape-key toggle + egui pause overlay
  camera.rs                     — Camera: world→NDC transform; HALF_VIEW constant lives here
  input.rs                      — InputSnapshot: per-player intent distilled from raw events
  player.rs                     — Player struct; tick_players / draw_players fns
  physics_layers.rs             — collision_layers / collision_mask presets for Body (walls, actors, projectiles)
  weapon.rs                     — WeaponStats, ProjectileBehavior, Weapon, Projectile / ProjectileMotion; spawn + integrate in world.rs
  world.rs                      — World: PhysicsWorld, tick, spawn by behavior, scripted integration, damage, cleanup
  hud.rs                        — FPS / backend / mouse-pos HUD helpers
  enemy/
    mod.rs                      — Enemy trait (actor, health, tick_ai, weapon_stats, projectile_behavior, draw)
    dummy.rs                    — DummyEnemy: stationary target that absorbs hits
  scenes/
    mod.rs                      — Scene trait + SceneTransition enum
    sandbox/
      mod.rs                    — SandboxScene: owns World + PauseState; Primary weapon + other sandbox egui tabs
    main_menu/
      mod.rs                    — MainMenuScene: title screen with Play / Quit
    level_select/
      mod.rs                    — LevelSelectScene: stub level picker
```

Future modules (not implemented now):

```
  pickup.rs                     — Pickup struct; spawn_pickup / tick_pickups / draw_pickups
  scenes/gameplay/              — full gameplay scene once sandbox graduates
  level.rs                      — static level geometry loaded into PhysicsWorld as Mesh bodies
```

---

## Core Types

### `InputSnapshot`  (`input.rs`)

Distilled per-player intent, computed once per tick from the raw `Vec<InputEvent>`.
This decouples all game logic from the raw event stream and is easy to construct
in tests without simulating real events.

```rust
/// Intent for one player slot for a single tick.
#[derive(Default, Clone, Copy)]
pub struct PlayerInput {
    pub move_dir: Vec2,      // normalised; zero when no input
    pub aim_dir:  Vec2,      // normalised; points toward cursor/right-stick
    pub fire:     bool,      // true on the tick the fire button was pressed
}

/// Intent for all four player slots, indexed by player index 0–3.
pub struct InputSnapshot([PlayerInput; 4]);

impl InputSnapshot {
    /// Consume raw events and produce a snapshot for this tick.
    pub fn from_events(events: &[InputEvent], cursor_ndc: Vec2) -> Self { ... }

    pub fn player(&self, idx: usize) -> PlayerInput { self.0[idx] }
}
```

**Keyboard/mouse always maps to P1** (slot 0), matching the existing convention.
Gamepads fill P1–P4 in connection order.

---

### `Player`  (`player.rs`)

```rust
pub struct Player {
    /// Index 0–3 — determines which InputSnapshot slot is read.
    pub slot:     usize,
    /// Handle into PhysicsWorld. Position is authoritative there.
    pub body:     BodyHandle,
    /// Direction the player is currently facing (for the aim line).
    pub facing:   Vec2,
    /// Hit-points remaining.
    pub health:   f32,
    /// Cosmetic color (used to tell players apart).
    pub color:    gfx::Color,
}
```

Player position is **not stored on `Player`** — it lives in the `Body` inside
`PhysicsWorld`. To read it: `world.physics.body(player.body).position`.

**Constructor:**
```rust
impl Player {
    pub fn new(slot: usize, start_pos: Vec2, physics: &mut PhysicsWorld) -> Self {
        let (collision_layers, collision_mask) = crate::physics_layers::player_collision();
        let body = physics.add_body(Body {
            position:    start_pos,
            velocity:    Vec2::ZERO,
            mass:        1.0,
            restitution: 0.3,
            collision_layers,
            collision_mask,
            collider:    Collider::Circle { radius: PLAYER_RADIUS },
        });
        Player { slot, body, facing: Vec2::X, health: 100.0, color: PLAYER_COLORS[slot] }
    }
}
```

**Systems (free functions, not methods):**
```rust
/// Apply input to each player's physics body velocity for this tick.
pub fn tick_players(players: &[Player], input: &InputSnapshot, physics: &mut PhysicsWorld);

/// Draw every live player.
pub fn draw_players(players: &[Player], physics: &PhysicsWorld, driver: &mut dyn GraphicsDriver, camera: &Camera);
```

Separating tick and draw means tests can call `tick_players` without a driver.

---

### `Camera`  (`camera.rs`)

The world→NDC conversion currently lives as ad-hoc `w()` / `wv()` functions
in `sandbox.rs`. Promote to a proper type so any system can convert coordinates
consistently.

```rust
pub struct Camera {
    /// World-space radius visible from centre to each viewport edge.
    pub half_view: f32,
}

impl Camera {
    pub fn world_to_ndc(&self, p: Vec2) -> Vec2 { p / self.half_view }
    pub fn ndc_to_world(&self, p: Vec2) -> Vec2 { p * self.half_view }
    pub fn scale(&self, world_len: f32) -> f32   { world_len / self.half_view }
}
```

`HALF_VIEW = 5.0` is the only initial value. Zoom / follow-cam can be added later.

---

### `World`  (`world.rs`)

The top-level container. Owns all arenas and the physics simulation.

```rust
pub struct World {
    pub physics: PhysicsWorld,
    pub players: Vec<Player>,
    // future: pub bullets: Vec<Bullet>,
    // future: pub pickups: Vec<Pickup>,
    pub camera:  Camera,

    // Timing (moved out of GameState)
    start:     std::time::Instant,
    last_tick: std::time::Instant,
    pub fps:   f32,

    // Raw cursor NDC (P1 mouse) — passed through to InputSnapshot
    pub cursor_ndc: Vec2,
}

impl World {
    pub fn new() -> Self { ... }

    /// Advance the simulation by one tick.
    ///
    /// 1. Build InputSnapshot from events.
    /// 2. Apply player input to physics velocities.
    /// 3. Step PhysicsWorld (collision + resolution).
    /// 4. (future) tick bullets, pickups, etc.
    pub fn tick(&mut self, events: Vec<InputEvent>) { ... }

    /// Draw the full scene.
    pub fn draw(&self, driver: &mut dyn GraphicsDriver) { ... }
}
```

`tick` calls systems in order; `draw` calls draw systems. Neither knows about
`App`, `winit`, or `wgpu`.

---

## Tick Pipeline (per frame)

```
App::run closure receives (world, events)
  │
  └─► world.tick(events)
        │
        ├─ 1. Build InputSnapshot::from_events(&events, cursor_ndc)
        ├─ 2. tick_players(&players, &snapshot, &mut physics)
        │       └─ sets body.velocity from move_dir * PLAYER_SPEED
        ├─ 3. physics.step(dt)
        │       └─ integrates positions; filters pairs by collision layers/mask; AABB; SAT; impulse resolution
        └─ 4. (future) tick_bullets, handle_pickups, ...

App::run closure receives (world, driver)
  │
  └─► world.draw(driver)        ← App already called begin_frame()
        │
        ├─ driver.clear(GROUND_COLOR)
        ├─ draw_grid(driver, &camera)
        ├─ draw_players(&players, &physics, driver, &camera)
        ├─ (future) draw_bullets, draw_pickups, ...
        └─ hud::draw_fps / draw_backend / draw_mouse_pos
```

---

## Sandbox Scene Migration

`sandbox::draw_scene` currently takes `(driver, fps, player_pos, cursor_ndc)`.
After the migration it becomes:

```rust
pub fn draw_scene(driver: &mut dyn GraphicsDriver, world: &World) { ... }
```

`main.rs` becomes:

```rust
App::run(
    World::new(),
    input_backend,
    |window| WgpuDriver::new(window),
    |world, events| { world.tick(events); },
    |world, driver| { game::sandbox::draw_scene(driver, world); },
);
```

The sandbox scene is the single rendering target during development. No new
scenes are added until the architecture is stable.

---

## Player Spawning (initial sandbox config)

For sandbox testing: spawn one player (slot 0) at world origin. Colour: blue
`0x2979FFFF` (matching the current hardcoded circle). When a second gamepad
connects, a second player can be spawned — this is future work.

```rust
impl World {
    pub fn new() -> Self {
        let mut w = World { physics: PhysicsWorld::new(), players: vec![], ... };
        w.players.push(Player::new(0, Vec2::ZERO, &mut w.physics));
        w
    }
}
```

---

## Constants

Move all shared constants to their canonical home:

| Constant | Value | Home |
|---|---|---|
| `HALF_VIEW` | `5.0` | `camera.rs` |
| `PLAYER_RADIUS` | `0.5` | `player.rs` |
| `PLAYER_SPEED` | `6.0` | `player.rs` |
| `PLAYER_COLORS` | `[blue, red, green, yellow]` | `player.rs` |
| `GROUND_COLOR` | `[0.13, 0.14, 0.12, 1.0]` | `sandbox.rs` |

---

## Test Plan

All tests use `SoftwareDriver` + `SimulatedBackend`. No GPU required.

### `player.rs` unit tests

| # | Test | What it checks |
|---|---|---|
| 1 | `player_moves_on_input` | After one tick with `move_dir = Vec2::X`, player body position has positive X. |
| 2 | `player_diagonal_not_faster` | Diagonal input (`move_dir = (1,1).normalize()`) produces the same speed as cardinal. |
| 3 | `player_stops_without_input` | Zero `move_dir` → velocity set to zero, position unchanged. |
| 4 | `two_players_independent` | P1 input does not move P2 body. |
| 5 | `players_collide` | Two players pushed toward each other separate after physics step. |

### `input.rs` unit tests

| # | Test | What it checks |
|---|---|---|
| 6 | `dpad_maps_to_move_dir` | DPad button events produce correct normalised `move_dir` on P1. |
| 7 | `stick_overrides_dpad` | Left-stick axis past dead-zone ignores DPad buttons. |
| 8 | `cursor_maps_to_aim` | `CursorMoved` event produces correct `aim_dir` relative to player position. |
| 9 | `gamepad_maps_to_correct_slot` | Events tagged with player index 1 affect slot 1, not slot 0. |

### `world.rs` integration tests

| # | Test | What it checks |
|---|---|---|
| 10 | `world_tick_advances_player` | Full `world.tick(events)` moves the player. |
| 11 | `world_draw_does_not_panic` | `world.draw(&mut software_driver)` completes without panic. |

### Snapshot regression

The existing `gfx_scene_snapshot` test targets `scene::draw_scene`. Update it
to call `sandbox::draw_scene(driver, &World::new())` so it exercises the new
path. Update the golden after the first intentional visual change, if any.

---

## Implementation Order

1. **`camera.rs`** — no dependencies; trivial to test.
2. **`input.rs`** — depends on `input` crate only; write tests first.
3. **`player.rs`** — depends on `physics` and `camera`; write tests first.
4. **`world.rs`** — assembles all of the above; write integration tests first.
5. **`sandbox.rs`** — update `draw_scene` signature; update snapshot golden if needed.
6. **`main.rs`** — update to pass `World` instead of `GameState`.
7. **Delete `state.rs`** — fully replaced by `world.rs` + `player.rs` + `input.rs`.

Each step is independently compilable and testable before moving to the next.

---

## Out of Scope (this phase)

- Bullets, projectiles, shooting mechanics
- Pickups / item drops
- Level geometry (static mesh bodies)
- Health / damage / respawn logic
- Multiple game scenes (menu, game over)
- Audio
- Networked play
