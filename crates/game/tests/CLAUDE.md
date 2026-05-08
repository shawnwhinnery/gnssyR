# CLAUDE.md — game/tests/

## Purpose

Integration and pixel regression tests for the `game` crate. All tests run headlessly — `SoftwareDriver` + `SimulatedBackend`; no GPU, no real window.

## Structure

```
tests/
  integration/
    main.rs                  — test entry point; all #[test] functions live here
    scenes/
      mod.rs                 — module root for test-only scenes
      gfx_showcase.rs        — GfxShowcaseScene: snapshot regression target
  snapshots/
    gfx_scene.bin            — golden pixel snapshot (committed to repo)
    gfx_scene.actual.bin     — written on mismatch only (for inspection; gitignored)
```

## Key Test: `gfx_scene_snapshot`

Defined in `integration/main.rs`. Runs `GfxShowcaseScene` for one frame using `SoftwareDriver::headless(512, 512)`, then pixel-compares against `snapshots/gfx_scene.bin`.

- Any pixel difference → test failure.
- **Regenerate** after an intentional visual change: `UPDATE_SNAPSHOTS=1 cargo test -p game`
- The golden file is 512 × 512 ARGB pixels stored as little-endian `u32` bytes.

## Test-Only Scenes (`integration/scenes/`)

Scenes here are exclusively for snapshot tests. They must **not** be re-exported from `game`'s library root or imported by production scenes. `GfxShowcaseScene` exercises every shape primitive, style variant, and transform helper defined in `gfx`.

## Adding Tests

- New snapshot scenes → `tests/integration/scenes/<name>.rs`
- New headless logic tests → new `#[test]` fn in `tests/integration/main.rs`
- Keep all tests `SoftwareDriver`-based; GPU-dependent tests are out of scope here.
