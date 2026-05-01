# Adding a New Hostile NPC Type

This guide walks you through the full process of creating a new enemy type that integrates with the existing physics, weapon, and damage systems.

---

## 1. Where things live

| Path | Purpose |
|------|---------|
| `crates/game/src/enemy/mod.rs` | `Enemy` trait definition |
| `crates/game/src/enemy/dummy.rs` | `Dummy` — reference implementation |
| `crates/game/src/world.rs` | `World::tick`, `spawn_enemy`, projectiles, damage resolution, cleanup |
| `crates/game/src/physics_layers.rs` | Layer/mask bitmasks and helpers for every `physics::Body` category |
| `crates/game/src/scenes/sandbox/mod.rs` | Sandbox egui: **Primary weapon** stats, enemies, inventory, forge/scrap as wired |
| `crates/game/src/weapon.rs` | `Weapon`, `WeaponStats`, `ProjectileBehavior`, `ProjectileOwner`, `Projectile` / `ProjectileMotion` |

---

## 2. The `Enemy` trait

```rust
pub trait Enemy {
    fn actor(&self) -> &ActorCore;
    fn actor_mut(&mut self) -> &mut ActorCore;
    fn body(&self) -> BodyHandle { self.actor().body }
    fn health(&self) -> f32;
    fn is_alive(&self) -> bool { self.health() > 0.0 }
    fn take_damage(&mut self, amount: f32);
    fn tick_ai(
        &mut self,
        dt: f32,
        player_positions: &[Vec2],
        physics: &mut PhysicsWorld,
    ) -> Vec<(Vec2, Vec<Vec2>)>;
    fn weapon_stats(&self) -> &WeaponStats;
    fn projectile_behavior(&self) -> ProjectileBehavior { ProjectileBehavior::Physics }
    fn loot_table(&self) -> LootTable;
    fn draw(&self, physics: &PhysicsWorld, driver: &mut dyn gfx::GraphicsDriver, camera: &Camera);
}
```

`projectile_behavior()` controls how this enemy’s shots move (same enum as the player weapon). Override when you want kinematic / seeking enemy ordnance; default keeps full **Physics** bodies for shots.

Every enemy must implement the non-default methods; `body`, `is_alive`, `projectile_behavior` have defaults you can rely on or override.

---

## 3. Minimum implementation checklist

### a) Create `crates/game/src/enemy/<name>.rs`

```rust
pub struct MyEnemy {
    pub body: BodyHandle,
    pub facing: Vec2,
    health: f32,
    weapon: Weapon,
}

impl MyEnemy {
    pub fn new(pos: Vec2, physics: &mut PhysicsWorld) -> Self {
        let (collision_layers, collision_mask) = crate::physics_layers::enemy_collision();
        let body = physics.add_body(Body {
            position: pos,
            velocity: Vec2::ZERO,
            mass: 1.0,
            restitution: 0.2,
            collision_layers,
            collision_mask,
            collider: Collider::Circle { radius: 0.5 },
        });
        Self {
            body,
            facing: Vec2::X,
            health: 80.0,
            weapon: Weapon::new(WeaponStats { /* ... */ }),
        }
    }
}
```

### b) Implement `Enemy`

Use **`dummy::Dummy`** (`crates/game/src/enemy/dummy.rs`) as the canonical template: `ActorCore`, `weapon_stats`, **`projectile_behavior`** (forwards `self.weapon.projectile_behavior`), `loot_table`, `tick_ai` returning `(origin, dirs)` volleys, and `draw`.

Older snippets that only show `fn body` / `fn weapon_stats` are incomplete — match the **current** `Enemy` trait in `enemy/mod.rs` (including `actor`, `loot_table`, and default `projectile_behavior` if you omit it).

### c) Re-export from `enemy/mod.rs`

```rust
pub mod my_enemy;
```

---

## 4. Giving the enemy a custom weapon

`WeaponStats` is a plain struct — construct one directly in `new()`:

```rust
weapon: Weapon::new(WeaponStats {
    fire_rate: 2.0,
    projectiles_per_shot: 3,
    shot_arc: 0.6, // radians
    burst_count: 1,
    burst_delay: 0.05,
    jitter: 0.05,
    kickback: 0.0,
    stability: 0.5,
    projectile_speed: 10.0,
    projectile_size: 0.08,
    projectile_lifetime: 2.5,
    piercing: 0,
    damage_total: 20.0,
    recoil_force: 0.0,
    ..WeaponStats::default()
})
```

Omitting fields is fine with `..WeaponStats::default()` for anything you are not customizing (oscillation, physics-projectile, rocket, seeking stats pick up defaults).

The caller (`World`) reads `weapon_stats()` and `projectile_behavior()` when spawning each shot.

---

## 5. How damage flows

```
World::tick()
  └─ player / enemy AI → spawn batches (`WeaponStats` + `ProjectileBehavior` snapshot per shot)
  └─ spawn projectiles (`Physics` → `add_body`; scripted → kinematic state only)
  └─ physics.step()            ← rigid bodies; layer/mask filter, AABB + SAT + impulses
  └─ physics-projectile friction & wall-bounce bookkeeping
  └─ integrate_scripted_projectiles()   ← Bullet / Rocket / Oscillating / Seeking
  └─ tick_projectiles() (lifetime)
  └─ resolve_damage()          ← contacts for physics shots; circle tests for scripted
  └─ cleanup_dead_enemies()    ← removes enemy bodies from PhysicsWorld
  └─ cleanup_projectiles()     ← uses **fresh** enemy handle list; try_body for overlap; TTL / walls / pierce rules
```

`ProjectileOwner::Enemy` is set automatically by `World` when spawning from an enemy volley. You do not set it yourself.

---

## 6. Wiring to the sandbox spawn button

Open `crates/game/src/world.rs` and extend `World::spawn_enemy`:

```rust
pub fn spawn_enemy(&mut self, pos: Vec2) {
    // Add a branch for your type, or replace the Dummy call.
    let enemy = MyEnemy::new(pos, &mut self.physics);
    self.enemies.push(Box::new(enemy));
}
```

The sandbox "Spawn Dummy" button already calls `spawn_enemy`. If you want a dedicated button for your type, add one to the "Enemies" egui panel in `SandboxScene::draw_ui` in `sandbox/mod.rs`, following the existing pattern.

---

## 7. Things to keep in mind

- **Physics body registration**: Always call `physics.add_body(...)` in the constructor and store the returned `BodyHandle`. Set `collision_layers` and `collision_mask` using `crate::physics_layers::enemy_collision()` (or the appropriate helper) so the enemy interacts with walls, players, other enemies, and the right projectile types. The `body()` method returns this handle; the World uses it for collision queries and cleanup.
- **Dead enemy cleanup**: When `is_alive()` returns `false`, `World::tick` removes the physics body and drops the `Box<dyn Enemy>` automatically. Do not remove it manually.
- **Borrow safety in `tick_ai`**: `&mut PhysicsWorld` is passed in, but `World` fields like `players` and `enemies` are iterated separately, so there is no double-borrow. Do not store a reference to `PhysicsWorld` across iterations.
- **Draw is `&self`**: No mutation in `draw`. Accumulate any state you need to render in `tick_ai`.
- **Projectile ownership**: Enemy bullets are orange-tinted (`#FF6666`) by default via the draw pass in `world.rs`. Player bullets remain white. You don't need to configure this — it's based on `ProjectileOwner`.
