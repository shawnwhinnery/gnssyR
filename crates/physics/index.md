# Spec: Physics

## Purpose

2D rigid-body physics for a local co-op game. Provides collision detection and impulse-based
resolution. No graphics dependency — the crate is pure math/logic built on `glam`.

Crate: `physics`.

---

## Collider

A `Collider` describes the shape of a body in its **local** (body) space:

| Variant | When to use |
|---------|-------------|
| `Circle { radius }` | Cheapest test — O(1) circle–circle; single distance comparison |
| `Convex { vertices }` | Convex polygon; SAT-based narrowphase |
| `Mesh { vertices, indices }` | Arbitrary triangle soup (e.g. static level geometry); broadphase-filtered SAT per triangle |

- `Collider::local_aabb()` returns the tight axis-aligned bounding box in local space.
- Vertices are in CCW winding order for `Convex`; triangle winding in `Mesh` is CCW.
- A `Circle` AABB is `[-r, r] × [-r, r]`.
- An empty `Convex` or `Mesh` collider produces degenerate/infinite AABB — callers must not create them.

---

## Aabb

Axis-aligned bounding box used for broadphase filtering.

- `Aabb::from_points(pts)` — tight box around a point set.
- `aabb.translate(offset)` — shifts min and max by offset.
- `aabb.overlaps(other)` — true iff the boxes share interior (touching edges → false).
- `aabb.expand(amount)` — grows min/max by `amount` on all sides (useful for sweep margin).

---

## Contact

Output of a narrowphase collision test.

```
Contact {
    normal: Vec2,  // unit vector from body A toward body B
    depth:  f32,   // penetration depth; positive means overlapping
    point:  Vec2,  // world-space contact point (approximate)
}
```

---

## Narrowphase: `narrow::detect`

```
detect(pos_a, col_a, pos_b, col_b) -> Option<Contact>
```

- Returns `None` when the shapes are separated.
- `Contact::normal` always points from A toward B.
- Dispatch table:

| A        | B        | Algorithm |
|----------|----------|-----------|
| Circle   | Circle   | Distance vs combined radii |
| Circle   | Convex   | SAT with nearest-vertex axis |
| Circle   | Mesh     | Per-triangle circle–convex; return shallowest contact |
| Convex   | Convex   | SAT over both edge-normal sets |
| Convex   | Mesh     | Per-triangle convex–convex SAT; return shallowest contact |
| Mesh     | Mesh     | Not supported (returns `None`) |

- Symmetric pairs (B–A) flip the contact normal.

### Circle–Circle
- `dist < r_a + r_b` → collision.
- Coincident centers produce `normal = Vec2::Y` as fallback.

### Circle–Convex (SAT)
- Test axes: one per edge normal, plus the axis from the nearest polygon vertex to the circle center.
- A separating axis on any test → `None`.
- Final normal is oriented from polygon centroid toward circle center.

### Convex–Convex (SAT)
- Test axes: edge normals of A, then edge normals of B.
- Minimum-overlap axis determines the contact normal and depth.
- Normal is oriented from A centroid toward B centroid.

### Mesh collisions
- AABB broadphase per triangle against the opposing shape's AABB.
- Narrowphase per surviving triangle.
- Among all colliding triangles, return the one with the **smallest** depth (shallowest penetration = most likely contact surface).
- Degenerate triangles (zero-length edges) are silently skipped.

---

## Body

```
Body {
    position:    Vec2,
    velocity:    Vec2,
    mass:        f32,        // f32::INFINITY for static/kinematic bodies
    restitution: f32,        // [0.0, 1.0] — bounciness coefficient
    collider:    Collider,
}
```

- `Body::is_static()` — true when `mass == f32::INFINITY`.
- Static bodies participate in collision detection but receive no impulse or positional correction.

---

## BodyHandle

```
BodyHandle(usize)
```

Opaque index into `PhysicsWorld`. Cheaply copyable. Using a handle after `remove_body` is a panic.

---

## PhysicsWorld

```
PhysicsWorld::new() -> Self
PhysicsWorld::add_body(body: Body) -> BodyHandle
PhysicsWorld::remove_body(handle: BodyHandle)
PhysicsWorld::body(handle) -> &Body
PhysicsWorld::body_mut(handle) -> &mut Body
PhysicsWorld::step(dt: f32)
PhysicsWorld::contacts() -> &[(BodyHandle, BodyHandle, Contact)]
```

### `step(dt)`

Executes one simulation tick:

1. **Integrate** — for each dynamic body: `position += velocity * dt`.
2. **Broadphase** — O(n²) AABB pair scan; discard non-overlapping pairs.
3. **Narrowphase** — `narrow::detect` on surviving pairs; collect contacts.
4. **Resolution** — for each contact:
   - Compute relative velocity along the normal.
   - Bodies already separating (`v_rel · n ≥ 0`) → skip.
   - Apply restitution impulse: `j = -(1 + e) * v_rel·n / (1/mA + 1/mB)`.
   - Distribute to velocities proportionally to inverse mass.
   - Apply Baumgarte positional correction (20 % of depth per step, with 0.01 slop).
5. Store resolved contacts for query via `contacts()`.

### `contacts()`
- Returns contacts from the **most recent** `step` call.
- Empty before the first `step`.

---

## Test Cases

- `circle_circle_no_overlap` — separated circles return `None`.
- `circle_circle_overlap` — overlapping circles return correct depth and normal.
- `circle_circle_coincident` — same center returns `normal = Vec2::Y`.
- `circle_convex_miss` — circle far from square returns `None`.
- `circle_convex_hit` — circle overlapping square edge returns correct normal and depth.
- `circle_convex_vertex_hit` — circle touching a polygon vertex uses the nearest-vertex axis.
- `convex_convex_miss` — two separated squares return `None`.
- `convex_convex_hit` — two overlapping squares return correct axis.
- `circle_mesh_miss` — circle above a mesh triangle returns `None`.
- `circle_mesh_hit` — circle penetrating a mesh triangle returns shallowest contact.
- `world_bounce` — two circles collide; after `step` velocities reverse along normal.
- `world_static` — dynamic circle vs static mesh; only dynamic body moves.
