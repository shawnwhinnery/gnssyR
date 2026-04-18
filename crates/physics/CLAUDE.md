# CLAUDE.md — physics

## Scope

`physics` is a pure-math 2D rigid-body physics layer. No graphics dependency — `glam` only.
It provides collision detection and impulse-based resolution for use by the `game` crate.

## Source of Truth

- Spec: `crates/physics/index.md`
- Key modules:
  - `crates/physics/src/collider.rs` — `Collider` enum (Circle, Convex, Mesh)
  - `crates/physics/src/aabb.rs` — AABB broadphase helper
  - `crates/physics/src/contact.rs` — `Contact` output type
  - `crates/physics/src/narrow.rs` — SAT dispatch and all narrowphase algorithms
  - `crates/physics/src/body.rs` — `Body` and `BodyHandle`
  - `crates/physics/src/world.rs` — `PhysicsWorld` simulation loop
- Tests: `crates/physics/tests/collision.rs`

## Non-Negotiable Invariants

- `Contact::normal` always points **from body A toward body B**. Every dispatch path in `narrow.rs` must preserve this — symmetric cases use `.map(flip)` on the correct side.
- `circle_convex` and `circle_mesh` return a normal pointing **from polygon toward circle** (polygon-centric). The dispatch table corrects orientation via `flip` so the public API remains A→B.
- Static bodies (`mass == f32::INFINITY`) receive zero impulse and zero positional correction. `inv_mass = 0.0` enforces this — never special-case it any other way.
- Mesh–Mesh collision is explicitly unsupported and returns `None`. Do not implement it without updating the spec.
- `BodyHandle` is an opaque index. Using one after `remove_body` is a documented panic, not undefined behaviour.

## Collision Normal Convention

```
detect(pos_a, col_a, pos_b, col_b) -> Option<Contact>
                                              ↑
                              normal points A → B
```

This means:
- `contact.normal.dot(vel_b - vel_a) < 0` → bodies are approaching (use for resolution gate).
- Impulse pushes A in `-normal` direction, B in `+normal` direction.
- When adding new shape pairs, verify orientation with a unit test before merging.

## Dispatch Table (narrow.rs)

| A      | B      | Inner call                          | Flip? |
|--------|--------|-------------------------------------|-------|
| Circle | Circle | `circle_circle(pos_a, ra, pos_b, rb)` | No (diff = pos_b−pos_a) |
| Circle | Convex | `circle_convex(pos_a, r, pos_b, verts)` | Yes (inner returns B→A) |
| Convex | Circle | `circle_convex(pos_b, r, pos_a, verts)` | No (inner returns A→B) |
| Circle | Mesh   | `circle_mesh(pos_a, r, pos_b, …)`   | Yes |
| Mesh   | Circle | `circle_mesh(pos_b, r, pos_a, …)`   | No |
| Convex | Convex | `convex_convex(pos_a, va, pos_b, vb)` | No (centroid orient) |
| Convex | Mesh   | `convex_mesh(pos_a, va, pos_b, …)`  | No |
| Mesh   | Convex | `convex_mesh(pos_b, vb, pos_a, …)`  | Yes |
| Mesh   | Mesh   | Not supported → `None`              | — |

## Editing Guidance

- When adding a new `Collider` variant, update `local_aabb()`, the dispatch table, and add tests for every new pair before touching any existing paths.
- SAT minimum-overlap tracking uses `< min_depth` (strict). First axis to achieve a given depth wins — keep this deterministic.
- `circle_convex` requires the nearest-vertex axis in addition to edge normals. Removing it breaks vertex-region collisions.
- Baumgarte constants (`CORRECTION_FACTOR = 0.2`, `SLOP = 0.01`) are in `world.rs`. Adjust carefully — too high causes jitter, too low causes sinking.
- Mesh narrowphase returns the shallowest contact among all triangles. This is intentional (most likely contact surface for level geometry).
- Do not add a graphics or `gfx` dependency to this crate.

## Validation

- Run crate tests: `cargo test -p physics`
- Run full workspace after any public API change: `cargo test`
