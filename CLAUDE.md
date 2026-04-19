# CLAUDE.md — gnssyR

## Project Overview

A local couch co-op 2D game (up to 4 players) with a web IO-game aesthetic. Written in Rust. The codebase is spec-driven: each crate has an `index.md` describing its intended behaviour, and tests are derived from those specs before implementation begins.

## Crate Map

See [crates/index.md](crates/index.md) for a one-line summary of every crate.

| Layer | Crates |
|-------|--------|
| Graphics abstraction | `gfx` |
| Graphics backends | `gfx-wgpu` (GPU/production), `gfx-software` (CPU/headless) |
| Input | `input` |
| Physics | `physics` (pure-math 2D rigid-body simulation, SAT narrowphase, impulse-based resolution) |
| App loop | `window` |
| Game logic | `game` (placeholder) |

## Workflow

1. Spec first — `crates/<name>/index.md` is the source of truth for a crate's behaviour.
2. Tests second — `crates/<name>/test.md` lists the test cases derived from the spec before any implementation is written.
3. Implementation last — code is written to make the tests pass.

## Key Conventions

- Tests use `SoftwareDriver` (no GPU) and `SimulatedBackend` (no hardware) so the full game loop is exercisable headlessly.
- `App` is generic over driver/input traits — it must not import concrete backends.
- Mesh handles are **frame-scoped**: do not cache across `begin_frame` boundaries.
- Axis deadzone is 0.1 (clamped to 0.0 in `GilrsBackend`).
- Keyboard/mouse always maps to P1; gamepads fill P1–P4 in connection order.

## Scene Snapshot Tests

`crates/game` contains a pixel-level regression test for the GFX showcase scene:

- **Test:** `crates/game/tests/integration/main.rs` — `gfx_scene_snapshot`
- **Golden file:** `crates/game/tests/snapshots/gfx_scene.bin` — committed to the repo; 512×512 ARGB pixels as little-endian `u32` bytes
- **Scene:** `GfxShowcaseScene` in `crates/game/tests/integration/scenes/gfx_showcase.rs` — exercises every shape primitive, style variant, and transform helper; test-only, not a production game scene
- **Test scenes folder:** `crates/game/tests/integration/scenes/` — place all snapshot-test-dedicated scenes here, separate from production scenes under `crates/game/src/scenes/`
- **Regenerate after intentional visual change:** `UPDATE_SNAPSHOTS=1 cargo test -p game`
- **On failure:** a `gfx_scene.actual.bin` is written next to the golden file for inspection


