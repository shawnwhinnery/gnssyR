# CLAUDE.md — game/src/scenes/

## Purpose

All production game scenes. Each scene is a self-contained state machine that owns simulation, rendering, and UI for one "screen." Scenes swap via `SceneTransition`, which is processed by `main.rs` between tick and render.

## Files

| File / Dir | Responsibility |
|-----------|---------------|
| `mod.rs` | `Scene` trait; `SceneTransition` enum |
| `main_menu/` | Title screen — see `main_menu/CLAUDE.md` |
| `level_select/` | Level select stub — see `level_select/CLAUDE.md` |
| `level1/` | Two-room level with phase state machine — see `level1/CLAUDE.md` |
| `sandbox/` | Dev sandbox with live parameter editing — see `sandbox/CLAUDE.md` |

## Scene Trait (`mod.rs`)

```rust
pub trait Scene {
    fn tick(&mut self, events: &[InputEvent]) -> Option<SceneTransition>;
    fn draw(&self, driver: &mut dyn gfx::GraphicsDriver);
    fn draw_ui(&self, _ctx: &egui::Context) {}   // default no-op
}

pub enum SceneTransition {
    Replace(Box<dyn Scene>),
    Quit,
}
```

Per-frame lifecycle (driven by `main.rs`):
1. `tick(events)` — advance simulation; return `Some(transition)` to change scene
2. If `Replace(next)` — old scene dropped via RAII, new scene installed
3. `draw(driver)` — gfx draw calls
4. `draw_ui(ctx)` — egui overlay (GPU path only; headless tests skip this)

## Production Scene Graph

```
MainMenuScene
  ├─→ Level1Scene         (Start Game)
  │     └─→ MainMenuScene (Return to Menu / Try Again)
  └─→ SandboxScene        (Start Sandbox)

LevelSelectScene           (stub — not yet reachable from MainMenuScene)
```

## Conventions

- **Test-only scenes** belong in `crates/game/tests/integration/scenes/` — never here.
- **`PauseState`** is a composable component; embed it in scenes that need pause rather than re-implementing per-scene.
- **egui deferred mutations**: flags set in `draw_ui` (via `Cell<bool>`) are consumed in the next `tick`. Never transition or mutate `World` directly from inside `draw_ui`.
- New production scenes go under `src/scenes/<name>/mod.rs`.
