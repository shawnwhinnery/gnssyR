# CLAUDE.md — game/src/enemy/

## Purpose

Hostile actor definitions. The `Enemy` trait provides a uniform interface for ticking AI, firing projectiles, reporting death, and producing loot. `Dummy` is the sole concrete enemy.

## Files

| File | Responsibility |
|------|---------------|
| `mod.rs` | `Enemy` trait; `LootTable` struct |
| `dummy.rs` | `Dummy`: circle body, orbit-and-close AI, fires via its own `Weapon` |

## Enemy Trait (`mod.rs`)

```rust
pub trait Enemy {
    fn actor(&self) -> &ActorCore;
    fn actor_mut(&mut self) -> &mut ActorCore;
    fn tick(&mut self, physics: &mut PhysicsWorld, players: &[Player], dt: f32) -> Vec<Projectile>;
    fn is_dead(&self) -> bool;
    fn loot_table(&self) -> LootTable;
    fn projectile_behavior(&self) -> ProjectileBehavior { ProjectileBehavior::Physics }
    fn draw(&self, physics: &PhysicsWorld, driver: &mut dyn GraphicsDriver, camera: &Camera);
}
```

`LootTable` fields:
- `min_drops` / `max_drops` — scrap count range on death
- `weapon_drop_chance` — probability [0, 1] of also producing a `WeaponDrop`

`World` drives enemies through `Enemy::tick` each frame and removes them when `is_dead()` is true, triggering loot generation.

## Dummy Enemy (`dummy.rs`)

- **Body**: circle collider; uses `enemy_collision()` layer preset from `physics_layers.rs`.
- **AI**: approaches nearest live player; maintains a preferred engagement distance before strafing.
- **Combat**: holds its own `Weapon`; fires when in range; returned `Vec<Projectile>` is inserted into `World::projectiles`.
- **Projectile behavior**: whatever `self.weapon.projectile_behavior` is (defaults to `ProjectileBehavior::Physics`).
- **Death**: `is_dead()` when `health <= 0`; `World` removes the physics body and spawns loot.
- **Spawning**: `World::spawn_enemy(pos)` constructs a `Dummy` with default weapon at `pos`.

## Adding New Enemies

1. Create `src/enemy/<name>.rs` implementing `Enemy`.
2. Re-export from `mod.rs`.
3. Add a spawn method to `World` if the scene needs to control placement.
4. Assign `enemy_collision()` from `physics_layers.rs` when constructing the physics body.
5. Return projectiles from `tick` — do not push directly to `World::projectiles`.
