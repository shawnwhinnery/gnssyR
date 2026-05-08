# CLAUDE.md — physics/src/

## Purpose

All 2D rigid-body physics. No graphics dependency — only `glam`. Collision detection (broadphase AABB + SAT narrowphase), impulse resolution, positional correction, and body management.

## Files

| File | Responsibility |
|------|---------------|
| `lib.rs` | Crate root — exports `PhysicsWorld`, `Body`, `BodyHandle`, `Collider`, `Contact`, `COLLISION_FILTER_MATCH_ALL` |
| `collider.rs` | `Collider` enum: `Circle { radius }`, `Convex { vertices: Vec<Vec2> }`, `Mesh { triangles }` |
| `aabb.rs` | `Aabb` type; `local_aabb(collider) -> Aabb` — axis-aligned bounding box used in broadphase |
| `contact.rs` | `Contact { normal: Vec2, depth: f32, point: Vec2 }` — narrowphase output; `normal` always points A→B |
| `narrow.rs` | `detect(pos_a, col_a, pos_b, col_b) -> Option<Contact>` — SAT dispatch table + all shape-pair algorithms |
| `body.rs` | `Body`: position, velocity, mass, `collision_layers`, `collision_mask`; `Body::collides_with`; `COLLISION_FILTER_MATCH_ALL = !0`; `BodyHandle` (opaque slot index) |
| `world.rs` | `PhysicsWorld`: body arena, `step(dt)` simulation pipeline, `contacts()`, `try_body` (non-panicking read after removal) |

## `step(dt)` Pipeline (`world.rs`)

```
1. Broadphase   — AABB overlap test for all body pairs
2. Layer filter — Body::collides_with: (A.layers & B.mask) && (B.layers & A.mask)
3. Narrowphase  — narrow::detect → Option<Contact>
4. Velocity res — impulse with Baumgarte velocity correction
5. Position cor — CORRECTION_FACTOR=0.2, SLOP=0.01 (world.rs constants)
6. Integration  — pos += vel * dt for all dynamic bodies
```

## Narrowphase Dispatch Table (`narrow.rs`)

| A      | B      | Inner fn | Flip result? |
|--------|--------|----------|-------------|
| Circle | Circle | `circle_circle` | No |
| Circle | Convex | `circle_convex` | Yes (inner returns B→A) |
| Convex | Circle | `circle_convex` (swapped) | No |
| Circle | Mesh | `circle_mesh` | Yes |
| Mesh | Circle | `circle_mesh` (swapped) | No |
| Convex | Convex | `convex_convex` | No |
| Convex | Mesh | `convex_mesh` | No |
| Mesh | Convex | `convex_mesh` (swapped) | Yes |
| Mesh | Mesh | unsupported → `None` | — |

## Critical Invariants

- `Contact::normal` points A→B in all public outputs — every dispatch path must preserve this.
- Static bodies: `mass = f32::INFINITY`, `inv_mass = 0.0` — never receive impulse or positional correction.
- Mesh–Mesh collision returns `None` — do not implement without updating the spec.
- `body(handle)` panics if the slot is empty (use only when the handle is known alive).
- `try_body(handle)` returns `None` if the slot is empty — use this in game code after `remove_body`.
- SAT minimum-overlap tracking uses strict `< min_depth` — keeps the algorithm deterministic.
- `circle_convex` requires the nearest-vertex axis in addition to edge normals — removing it breaks vertex-region contacts.
- Never add a `gfx` or graphics crate dependency here.
