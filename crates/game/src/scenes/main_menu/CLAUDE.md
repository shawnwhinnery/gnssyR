# CLAUDE.md — game/src/scenes/main_menu/

## Purpose

The title screen. First scene the player sees. Routes to the main game, the sandbox, or exits.

## File

`mod.rs` — entire scene (small; no physics, no `World`).

## Behaviour

- Renders a full-screen `egui::CentralPanel` with the game title and action buttons.
- **Start Game** → `SceneTransition::Replace(Box::new(Level1Scene::new()))`
- **Start Sandbox** → `SceneTransition::Replace(Box::new(SandboxScene::new()))`
- **Quit** → `SceneTransition::Quit`
- Enter / Space also triggers Start Game.

## Constraints

- No `World`, no physics, no `PauseState` — purely a UI scene.
- `draw` is a no-op; all visual content is in `draw_ui`.
- Keep this scene lightweight — it is reconstructed each time the player returns to menu.
