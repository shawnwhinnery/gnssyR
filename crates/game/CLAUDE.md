# CLAUDE.md — game

## Scope

`game` contains gameplay-facing logic and scenes built on top of `gfx`, `input`, and `window`. The gameplay spec is still evolving, but rendering/test infrastructure already exists.

## Source of Truth

- Current spec placeholder: `crates/game/game.md`
- Integration tests: `crates/game/tests/integration/main.rs`
- Production scenes: `crates/game/src/scenes/` — `Scene` trait + `SandboxScene`
- Test-only scenes: `crates/game/tests/integration/scenes/` — scenes used exclusively for snapshot tests (e.g. `GfxShowcaseScene`)
- Runtime glue: `crates/game/src/main.rs`, `crates/game/src/lib.rs`

## Current Project Guarantees

- Supports headless/integration testing using `SoftwareDriver` + `SimulatedBackend`.
- Snapshot regression test protects visual output of the GFX showcase scene.
- Golden snapshot file lives at `crates/game/tests/snapshots/gfx_scene.bin`.
- Set `UPDATE_SNAPSHOTS=1` when intentionally updating scene visuals.

## Editing Guidance

- Keep game-side code compatible with both headless tests and interactive runtime.
- Production scenes (`src/scenes/`) are used by both the runtime and integration tests. Test-only scenes (`tests/integration/scenes/`) are used exclusively by snapshot tests — do not import them from the library crate.
- Treat snapshot diffs as signal: confirm intentional visual changes before updating goldens.
- As gameplay spec solidifies, promote requirements from `game.md` into concrete tests first.

## Validation

- Run game tests: `cargo test -p game`
- For intentional visual updates: `UPDATE_SNAPSHOTS=1 cargo test -p game`
