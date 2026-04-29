# Adding a New Hostile NPC Type

This guide walks you through the full process of creating a new enemy type that integrates with the existing physics, weapon, and damage systems.

---

## 1. Where things live

| Path | Purpose |
|------|---------|
| `crates/game/src/enemy/mod.rs` | `Enemy` trait definition |
| `crates/game/src/enemy/dummy.rs` | `Dummy` — reference implementation |
| `crates/game/src/scenes/sandbox/world.rs` | `World::spawn_enemy`, damage resolution, cleanup |
| `crates/game/src/scenes/sandbox/mod.rs` | Sandbox UI (spawn buttons) |
| `crates/game/src/weapon.rs` | `Weapon`, `WeaponStats`, `ProjectileOwner` |

---

## 2. The `Enemy` trait

```rust
pub trait Enemy {
    fn body(&self) -> BodyHandle;
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
    fn draw(&self, physics: &PhysicsWorld, driver: &mut dyn gfx::GraphicsDriver, camera: &Camera);
}
```

Every enemy must implement all six methods. The provided defaults (`is_alive`) are optional to override.

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
        let body = physics.add_body(Body {
            position: pos,
            velocity: Vec2::ZERO,
            mass: 1.0,
            restitution: 0.2,
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

```rust
impl Enemy for MyEnemy {
    fn body(&self) -> BodyHandle { self.body }
    fn health(&self) -> f32 { self.health }
    fn take_damage(&mut self, amount: f32) {
        self.health = (self.health - amount).max(0.0);
    }
    fn weapon_stats(&self) -> &WeaponStats { &self.weapon.stats }

    fn tick_ai(
        &mut self,
        dt: f32,
        player_positions: &[Vec2],
        physics: &mut PhysicsWorld,
    ) -> Vec<(Vec2, Vec<Vec2>)> {
        let my_pos = physics.body(self.body).position;

        // — AI logic here: update self.facing, set velocity —
        physics.body_mut(self.body).velocity = self.facing * MY_SPEED;

        // Decide whether to fire.
        let fire_intent = /* condition */;
        let volleys = self.weapon.tick(dt, fire_intent);
        if volleys > 0 {
            let dirs = self.weapon.volley_directions(self.facing);
            vec![(my_pos, dirs)]
        } else {
            vec![]
        }
    }

    fn draw(&self, physics: &PhysicsWorld, driver: &mut dyn gfx::GraphicsDriver, camera: &Camera) {
        if !self.is_alive() { return; }
        // — draw using gfx primitives —
    }
}
```

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
    shot_arc: 0.6,       // radians
    jitter: 0.05,
    projectile_speed: 10.0,
    projectile_size: 0.08,
    projectile_lifetime: 2.5,
    damage_total: 20.0,
    recoil_force: 0.0,   // enemies typically don't need recoil
    ..WeaponStats::default()
})
```

The caller (`World`) reads `weapon_stats()` to configure the spawned `Projectile`. No additional wiring is needed.

---

## 5. How damage flows

```
World::tick()
  └─ enemy.tick_ai() → returns (origin, dirs) spawn requests
  └─ World spawns Projectile { owner: ProjectileOwner::Enemy, damage, … }
  └─ physics.step()
  └─ resolve_damage()
       ├─ Enemy projectile ∩ player body  → player.health -= damage
       └─ Player projectile ∩ enemy body → enemy.take_damage(damage)
  └─ cleanup_dead_enemies()   ← removes body from PhysicsWorld
  └─ cleanup_projectiles()    ← despawns on wall/enemy hit or lifetime expiry
```

`ProjectileOwner::Enemy` is set automatically by `World::spawn_enemy`. You do not need to set it yourself.

---

## 6. Wiring to the sandbox spawn button

Open `crates/game/src/scenes/sandbox/world.rs` and extend `World::spawn_enemy`:

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

- **Physics body registration**: Always call `physics.add_body(...)` in the constructor and store the returned `BodyHandle`. The `body()` method returns this handle; the World uses it for collision queries and cleanup.
- **Dead enemy cleanup**: When `is_alive()` returns `false`, `World::tick` removes the physics body and drops the `Box<dyn Enemy>` automatically. Do not remove it manually.
- **Borrow safety in `tick_ai`**: `&mut PhysicsWorld` is passed in, but `World` fields like `players` and `enemies` are iterated separately, so there is no double-borrow. Do not store a reference to `PhysicsWorld` across iterations.
- **Draw is `&self`**: No mutation in `draw`. Accumulate any state you need to render in `tick_ai`.
- **Projectile ownership**: Enemy bullets are orange-tinted (`#FF6666`) by default via the draw pass in `world.rs`. Player bullets remain white. You don't need to configure this — it's based on `ProjectileOwner`.
