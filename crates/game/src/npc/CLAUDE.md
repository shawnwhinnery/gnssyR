# CLAUDE.md — game/src/npc/

## Purpose

Friendly non-player characters the player can interact with. Currently contains only the Forgemaster, who mediates the forge crafting system.

## Files

| File | Responsibility |
|------|---------------|
| `mod.rs` | `FriendlyNpc` trait; `NpcKind` enum |
| `forgemaster.rs` | `Forgemaster`: amber hexagon, static physics body, forge interaction zone |

## FriendlyNpc Trait (`mod.rs`)

```rust
pub trait FriendlyNpc {
    fn actor(&self) -> &ActorCore;
    fn body(&self) -> BodyHandle;           // default impl: actor().body
    fn interaction_radius(&self) -> f32;
    fn kind(&self) -> NpcKind;
    fn draw(&self, physics: &PhysicsWorld, driver: &mut dyn GraphicsDriver, camera: &Camera);
}

pub enum NpcKind { Forgemaster }
```

`World::nearest_interactable_npc()` returns `Option<NpcKind>` by scanning all NPCs for proximity to any live player within their `interaction_radius`. Scenes use this to display prompt UI.

## Forgemaster (`forgemaster.rs`)

- **Visual**: amber hexagon drawn at its world position.
- **Physics**: infinite-mass static body — uses `npc_collision()` preset from `physics_layers.rs`.
- **Interaction radius**: 1.8 world units.
- **Spawning**: `World::spawn_forgemaster(pos)`.
- **Forge dialog**: `SandboxScene` detects proximity each tick and shows `[E] Forgemaster` prompt; pressing E sets a `Cell<bool>` flag that opens the forge dialog in the next `draw_ui` pass.

## Adding New NPCs

1. Create `src/npc/<name>.rs` implementing `FriendlyNpc`.
2. Add a variant to `NpcKind`.
3. Re-export from `mod.rs`.
4. Add a `World::spawn_<name>(pos)` method.
5. Use `npc_collision()` from `physics_layers.rs` when constructing the physics body.
