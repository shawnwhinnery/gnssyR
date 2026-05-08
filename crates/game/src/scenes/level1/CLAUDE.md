# CLAUDE.md — game/src/scenes/level1/

## Purpose

The primary playable level. Two-room layout with a locked door and a phase-based enemy wave system. The only scene with a win condition today.

## File

`mod.rs` — entire scene.

## Composition

`Level1Scene` holds:
- `world: World` — all simulation (physics, players, enemies, projectiles, scraps)
- `pause: PauseState` — Escape toggle; egui pause overlay
- `phase: Phase` — current progression state

## Phase State Machine

```
Room1
  → (all Room1 enemies dead)  → DoorOpen     (World::remove_wall on door body)
  → (player crosses threshold) → Room2        (World::add_wall reinstates door)
  → (all Room2 enemies dead)  → Win
  → (all players dead at any phase) → GameOver
```

Transitions are evaluated in `tick`. Win / GameOver show an egui overlay with retry / menu options.

## Door Mechanic

The door is a `Wall` registered via `World::add_wall` at scene construction. `World::remove_wall` is called on the door's handle when the `DoorOpen` phase begins. `World::add_wall` re-registers it when `Room2` begins to block retreat.

## egui Overlays (`draw_ui`)

- P1 health bar — always visible during play
- Phase banner — shown on Win / GameOver with retry and return-to-menu buttons
- Pause overlay — via `PauseState::draw_ui`

## Deferred Flags

`restart` and `return_to_menu` are `Cell<bool>` fields set from `draw_ui` and consumed at the top of `tick`. On restart: `respawn_player(P1)` and reset to `Room1`. On return: `SceneTransition::Replace(MainMenuScene)`.

## Enemy Spawn Layout

Spawn positions are hardcoded per phase in `Level1Scene::new()`. Room1 and Room2 each have distinct spawn points set before the scene is entered.
