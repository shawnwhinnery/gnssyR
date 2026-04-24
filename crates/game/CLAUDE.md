# CLAUDE.md — game

## Scope

`game` contains gameplay-facing logic and scenes built on top of `gfx`, `input`, and `window`. The gameplay spec is still evolving, but rendering/test infrastructure already exists.

## Source of Truth

- Current spec placeholder: `crates/game/game.md`
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
| `SandboxScene` | `src/scenes/sandbox/` | Current only scene; owns `World` + `PauseState` |

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

## Validation

- Run game tests: `cargo test -p game`
- For intentional visual updates: `UPDATE_SNAPSHOTS=1 cargo test -p game`
