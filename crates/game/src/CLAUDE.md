# CLAUDE.md — game/src/

## Module Map

Every module in `game/src/` and what it is solely responsible for:

| File / Dir | Primary concern |
|-----------|----------------|
| `lib.rs` | Crate root — module declarations and public re-exports |
| `main.rs` | Binary entry point — creates `WgpuDriver`, starts `App::run_with_ui` with `MainMenuScene` |
| `actor.rs` | `ActorCore` (shared `BodyHandle` + `facing`); `Actor` trait; `draw_shape` free function |
| `camera.rs` | `Camera`: world↔NDC conversion, smooth-follow spring, `HALF_VIEW = 7.28` world units |
| `hud.rs` | Debug overlay: FPS counter, backend name, cursor position, collision hit count |
| `input.rs` | `InputState` (per-player hold-state map), `InputSnapshot` (single-tick button reads) |
| `mode.rs` | `GameMode` enum — `Playing` / `Paused` |
| `pause.rs` | `PauseState`: Escape-toggle, `Cell<GameMode>`, egui pause modal |
| `physics_layers.rs` | Collision bitmask presets for walls, players, enemies, NPCs, projectiles |
| `player.rs` | `Player`: slot, `ActorCore`, health, color, `Weapon`, `weapon_name` |
| `weapon.rs` | `WeaponStats`, `ProjectileBehavior`, `Weapon`, `Projectile`, firing state machine |
| `world.rs` | `World`: top-level simulation container; orchestrates all subsystems |
| `scrap.rs` | `Scrap`, `ScrapColor` (8 variants), `ScrapShape` (4 variants), `Inventory`, collection logic |
| `mod_part.rs` | `ModPart` forge artifact; `forge()` weighted-blend algorithm; `draw_mod_part()` |
| `loot.rs` | ⚠ **Under active refactor** — weapon drop generation; do not read/modify without checking |
| `enemy/` | `Enemy` trait, `LootTable`, `Dummy` — see `enemy/CLAUDE.md` |
| `namegen/` | Procedural gun + mod name generation — see `namegen/CLAUDE.md` |
| `npc/` | `FriendlyNpc` trait, `Forgemaster` — see `npc/CLAUDE.md` |
| `scenes/` | `Scene` trait, all production scenes — see `scenes/CLAUDE.md` |

## Data Flow (one frame)

```
main.rs → App::run_with_ui
  input.poll()                       → [InputEvent]
  scene.tick([InputEvent])
    └─ world.tick(dt, events)
         ├─ physics.step(dt)
         ├─ player[*].weapon.tick(dt, fire_intent)  → volley count
         ├─ world spawns projectiles from volleys
         ├─ projectile overlaps → damage / despawn
         ├─ scrap / weapon-drop collection checks
         └─ camera.update(avg_live_player_pos, dt)
  scene.draw(driver)                 → gfx draw calls
  scene.draw_ui(ctx)                 → egui overlay
```

## Shared Invariants

- `ActorCore` is the common substrate for `Player`, `Dummy`, and all `FriendlyNpc` — every live entity in the world carries `BodyHandle + facing`.
- **Always** use collision presets from `physics_layers.rs`; never set raw bitmask literals inline.
- `PauseState.mode` must be `Cell<GameMode>` so `draw_ui(&self)` can toggle it without `&mut self`.
- `WeaponStats::defaults()` is `const fn` — preserve this constraint; it is needed for static loot table ranges.
- All draw positions must pass through `Camera::world_to_ndc` — never draw in raw world coordinates.
- Deferred mutations from `draw_ui` use `Cell<bool>` flags (e.g. `restart`, `forge_requested`) consumed in the next `tick`.
