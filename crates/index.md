# Crates Index

| Crate | Description |
|-------|-------------|
| `gfx` | Backend-agnostic vector graphics layer. Defines the `GraphicsDriver` trait, core types (`Color`, `Transform`), path/shape primitives, and the scene graph. Consumed by both concrete driver crates. |
| `gfx-wgpu` | GPU-backed `GraphicsDriver` using `wgpu`. Production driver targeting Vulkan (Linux/Steam Deck), Metal (macOS), and DX12 (Windows). |
| `gfx-software` | CPU-based `GraphicsDriver` with no GPU or display requirement. Used for headless testing and CI. Exposes a pixel buffer for assertion. |
| `input` | Unified input abstraction for up to 4 local players. Normalises gamepads and keyboard/mouse into a single `InputEvent` stream. Includes `SimulatedBackend` for tests. |
| `window` | `App::run` entry point. Owns the winit event loop, drives the per-frame tick/render sequence, and coordinates the input backend and graphics driver. |
| `physics` | 2D rigid-body physics. Impulse-based resolution with SAT narrowphase; **collision layers** (`Body::collision_layers` / `collision_mask`) filter pairs before broadphase. **`PhysicsWorld::try_body`** for non-panicking access after `remove_body`. Circle, Convex, and triangle Mesh colliders. |
| `game` | Core game loop: `World`, `SandboxScene` (egui sandbox: **Primary weapon** tab for `WeaponStats` + `ProjectileBehavior`, plus enemies / inventory), `Weapon`, `Projectile`, `ProjectileMotion`, and `physics_layers`. Couch co-op target (up to 4 players). |
