# CLAUDE.md — game/src/scenes/sandbox/

## Purpose

Developer sandbox for live-editing game parameters and testing mechanics in isolation. Not a production level — a scratchpad with full simulation and extensive egui tooling.

## File

`mod.rs` — entire scene (~1200 lines).

## Composition

`SandboxScene` holds:
- `world: World` — full simulation
- `pause: PauseState` — Escape toggle

## egui Panel Structure

The **Sandbox** floating `egui::Window` contains three tabs:

| Tab | What it controls |
|-----|-----------------|
| **Primary weapon** | Live-edits all `WeaponStats` fields + `ProjectileBehavior` selector via `weapon_editor(ui, stats, behavior)` |
| **Enemies** | Spawn `Dummy` enemies at a target position; choose enemy weapon behavior |
| **Inventory** | View `Inventory` scrap counts by color × shape; open the Forge dialog |

## Forge Dialog

Triggered when the player presses E within 1.8 world units of the Forgemaster NPC:
1. `World::nearest_interactable_npc()` returns `Some(NpcKind::Forgemaster)` in `tick`
2. E key sets `forge_requested: Cell<bool>` 
3. `draw_ui` reads the flag and shows the forge contribution UI
4. On confirm: `forge(contributions)` → `Option<ModPart>` stored on `SandboxScene`
5. `draw_mod_part` renders the result near the player each frame

## Wall Layout

`add_sandbox_walls(world)` builds the perimeter and any interior dividers at scene construction. All walls use `wall_collision()` layer preset. No dynamic door mechanic — walls are static for the session.

## Slow-Motion Mode

`World::time_scale` can be tuned via an egui slider (default 1.0; useful range 0.1–1.0) for debugging projectile trajectories.

## Invariants

- Sandbox is a dev tool; it must never be listed as a playable level.
- All deferred mutations from `draw_ui` use `Cell<bool>` or `Cell<T>` consumed in `tick`.
- `enemy_spawn_requests` accumulates spawn positions queued in `draw_ui`; `tick` drains it.
- `SandboxScene` is started from `MainMenuScene` via "Start Sandbox".
