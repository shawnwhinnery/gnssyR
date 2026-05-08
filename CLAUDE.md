# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A local couch co-op 2D game (up to 4 players) with a web IO-game aesthetic. Written in Rust. The codebase is spec-driven: each crate has an `index.md` describing its intended behaviour, and tests are derived from those specs before implementation begins.

## Commands

```sh
cargo run                              # run the game (requires display)
cargo test                             # all tests (headless — no GPU required)
cargo test -p <crate>                  # single crate, e.g. -p game, -p physics
cargo test -p game -- <test_name>      # single test by name
cargo check                            # fast compile check without linking
UPDATE_SNAPSHOTS=1 cargo test -p game  # regenerate pixel snapshot golden file
```

## Crate Map

See [crates/index.md](crates/index.md) for one-line summaries.

| Layer | Crates |
|-------|--------|
| Graphics abstraction | `gfx` |
| Graphics backends | `gfx-wgpu` (GPU/production), `gfx-software` (CPU/headless) |
| Input | `input` |
| Physics | `physics` |
| App loop | `window` |
| Game logic | `game` |
| UI | egui 0.29 (immediate-mode, rendered by `gfx-wgpu` on top of game content) |

### Dependency Rules

```
game        → gfx, input, physics, window, gfx-software (dev), egui
window      → gfx, input, winit, gfx-wgpu (feature-gated)
gfx-wgpu    → gfx, wgpu, egui-wgpu
gfx-software → gfx
input       → gilrs (optional)
physics     → glam only — no gfx dependency ever
gfx         → glam only — no backend crates ever
```

`App` in `window` is generic over driver/input traits and must not import concrete backends.

## Workflow

1. Spec first — `crates/<name>/index.md` is the source of truth for a crate's behaviour.
2. Tests second — `crates/<name>/test.md` lists the test cases derived from the spec before any implementation is written.
3. Implementation last — code is written to make the tests pass.

## Key Conventions

- Tests use `SoftwareDriver` (no GPU) and `SimulatedBackend` (no hardware) so the full game loop is exercisable headlessly.
- Mesh handles are **frame-scoped**: do not cache across `begin_frame` boundaries.
- Axis deadzone is 0.1 (clamped to 0.0 in `GilrsBackend`).
- Keyboard/mouse always maps to P1; gamepads fill P1–P4 in connection order.
- **`PhysicsWorld::try_body`** — use this (not `body()`) in game code after `remove_body`; returns `None` if the slot is empty. `body()` panics on a removed handle.
- Collision layer presets live in `physics_layers.rs` — never set raw bitmask literals inline.

## Scene Management

All live game state lives in a single `Box<dyn game::scenes::Scene>` owned by the main loop. Scenes drive themselves through three methods:

| Method | Signature | Purpose |
|--------|-----------|---------|
| `tick` | `(&mut self, events) -> Option<SceneTransition>` | Advance simulation one logical step |
| `draw` | `(&self, driver)` | Render gfx content |
| `draw_ui` | `(&self, ctx)` | Render egui overlay (default no-op) |

`SceneTransition::Replace(Box<dyn Scene>)` swaps the active scene; `SceneTransition::Quit` signals exit.

### Production scene graph

```
MainMenuScene
  ├─→ Level1Scene        (Start Game)
  │     └─→ MainMenuScene (Return to Menu / Game Over)
  └─→ SandboxScene       (Start Sandbox)
LevelSelectScene         (stub, not yet reachable from MainMenuScene)
```

### `draw_ui` deferred mutation pattern

`draw_ui` receives `&self`, so it cannot mutate `World` or trigger transitions directly. The pattern throughout all scenes:

1. Flags are `Cell<bool>` (or `Cell<T>`) fields on the scene struct.
2. `draw_ui` sets flags via `Cell::set`.
3. The next `tick` reads and clears those flags, then performs the actual mutation.

Never transition or mutate `World` from inside `draw_ui`. See `PauseState` in `pause.rs` as the canonical reference implementation.

### `PauseState` composition

Embed `PauseState` in any scene that needs a pause menu:

```rust
self.pause.tick(events);           // toggles Playing↔Paused on Escape
if !self.pause.is_paused() {
    self.world.tick(events);
}
// in draw_ui:
self.pause.draw_ui(ctx);
```

`PauseState.mode` is `Cell<GameMode>` so `draw_ui` can write back from `&self`.

## UI (egui)

- `App::run_with_ui` — manages the egui frame lifecycle; use this in `main.rs` instead of `App::run`.
- `Scene::draw_ui(&self, ctx: &egui::Context)` — override to render egui content; default is a no-op.
- `draw_ui` is called after `scene.draw(driver)` each frame.
- `GraphicsDriver` has no egui coupling — `gfx` and `gfx-software` are egui-free.
- `SoftwareDriver` (headless tests) does not implement `EguiRenderer`; tests via `App::run_frames` skip the egui path entirely.
- egui input events are consumed before game input; game input is skipped when egui reports `consumed`.

## Scene Snapshot Tests

- **Test:** `crates/game/tests/integration/main.rs` — `gfx_scene_snapshot`
- **Golden file:** `crates/game/tests/snapshots/gfx_scene.bin` — committed; 512×512 ARGB pixels as little-endian `u32` bytes
- **Regenerate after intentional visual change:** `UPDATE_SNAPSHOTS=1 cargo test -p game`
- **On failure:** `gfx_scene.actual.bin` is written next to the golden file for inspection
- Test-only scenes belong in `crates/game/tests/integration/scenes/` — never in `src/scenes/`

## Active Refactor: `loot.rs`

`crates/game/src/loot.rs` is under active refactor. Do not modify it without checking the current state of the file first. The game/CLAUDE.md and game/src/CLAUDE.md both note this with a warning.
