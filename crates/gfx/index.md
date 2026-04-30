# gfx

Backend-agnostic vector graphics layer. Defines the `GraphicsDriver` trait and all shared types consumed by both concrete driver crates (`gfx-wgpu`, `gfx-software`).

---

## GraphicsDriver Trait Contract

- `begin_frame` must be called exactly once before any draw call in a frame
- `end_frame` must be called exactly once after all draw calls in a frame
- `present` must be called after `end_frame`
- Calling draw methods outside of a `begin_frame` / `end_frame` pair is undefined behaviour

### `clear(color)`
- Overwrites every pixel in the render target with `color`
- Must be called after `begin_frame`
- Calling `clear` multiple times in a frame overwrites the previous clear

### `upload_mesh(vertices, indices) -> MeshHandle`
- Returns a handle valid for the remainder of the current frame
- An empty vertex or index slice produces an unspecified but valid handle
- Handles from a previous frame must not be used after the next `begin_frame`

### `draw_mesh(handle, transform, color)`
- Renders the mesh identified by `handle` with the given affine transform and tint color
- `color` is multiplied with per-vertex color data
- Passing an invalid or expired handle is undefined behaviour
- Multiple calls with the same handle are valid; each is an independent draw


### `upload_texture(pixels, width, height) -> TextureHandle`
- `pixels` are packed ARGB `u32` in row-major order (`(a << 24) | (r << 16) | (g << 8) | b`)
- Returns a handle that remains valid across `begin_frame` / `end_frame` until `free_texture`
- `width` and `height` must be positive; `pixels.len()` must equal `width * height` (otherwise undefined)
- Texture handles are independent of mesh handles (no numeric relationship is guaranteed)

### `free_texture(handle)`
- Releases resources for a texture created with `upload_texture`
- Freeing an unknown or already-freed handle is a no-op

### `draw_bitmap(texture, transform, tint)`
- Draws the entire bitmap as a quad in clip space with the same transform semantics as `draw_mesh`
- `tint` multiplies sampled texel RGBA (straight alpha)
- Invalid or freed texture handles result in no draw (no panic)

### `surface_size() -> (u32, u32)`
- Returns the current pixel dimensions of the render target
- May change between frames (e.g. on window resize)
- Width and height are both > 0 when a valid surface exists

---

## Core Types

### `Color`
- `Color::rgba(r, g, b, a)` constructs a colour with all components in [0, 1]
- `Color::hex(0xRRGGBBAA)` parses a packed hex value
- `Color::with_alpha(a)` returns a copy with only the alpha changed
- `Color::to_array()` returns `[r, g, b, a]` as `[f32; 4]`
- Named constants: `BLACK`, `WHITE`, `TRANSPARENT`

### `Transform`
- `Transform::identity()` is the no-op transform
- `identity.apply(p) == p` for any point `p`
- `Transform::translate(x, y).apply(Vec2::ZERO) == Vec2::new(x, y)`
- `Transform::rotate(θ).apply(Vec2::new(1, 0))` ≈ `Vec2::new(cos θ, sin θ)`
- `Transform::scale(sx, sy).apply(Vec2::new(1, 1)) == Vec2::new(sx, sy)`
- `a.then(b).apply(p) == b.apply(a.apply(p))` (composition order)
- `t.then(t.inverse().unwrap())` ≈ `Transform::identity()` for invertible `t`

---

## Path Builder

- A newly constructed `PathBuilder` with no segments produces an open, empty `Path`
- `move_to` does not add a visible segment; it sets the current pen position
- `line_to` adds a straight segment from the current point
- `quad_to(cp, end)` adds a quadratic bezier
- `cubic_to(cp1, cp2, end)` adds a cubic bezier
- `arc_to(center, radius, start, end)` adds an arc with the given angular extent
- `build()` produces an **open** path
- `close()` produces a **closed** path; subsequent tessellation connects the last point back to the first

---

## Parametric Path API

All `t` values are arc-length parameterised over [0, 1].

- `path.point_at(0.0)` returns the start point
- `path.point_at(1.0)` returns the end point (or start point if closed)
- `path.tangent_at(t)` returns a unit vector (length ≈ 1.0)
- `path.normal_at(t)` is perpendicular to `tangent_at(t)` (dot product ≈ 0)
- `path.length()` returns a positive value for any non-degenerate path
- `path.split_at(0.5)`: the two halves together reconstruct the original path
- `path.trim(0.25, 0.75).length() ≈ path.length() * 0.5`
- `path.reverse().point_at(0.0) == path.point_at(1.0)`
- `path.offset(d)`: every point on the result is approximately `d` units from the original path

---

## Shape Primitives

All constructors return a closed `Path` unless noted.

### `circle(center, radius)`
- All points on the path are within `ε` of `radius` from `center`
- `path.is_closed() == true`

### `ellipse(center, rx, ry)`
- Points satisfy `(x/rx)² + (y/ry)² ≈ 1` in local space

### `rect(origin, size)`
- Bounding box equals `Rect { origin, size }`
- Exactly 4 corners

### `rounded_rect(origin, size, r)`
- `r == 0` produces the same shape as `rect`
- Corner points are offset by `r` from the straight-edge intersections

### `regular_polygon(center, radius, sides)`
- All vertices are equidistant from `center` at distance `radius`
- Interior angles are equal
- Requires `sides >= 3`

### `star(center, outer, inner, points)`
- Alternates between `outer` and `inner` radius vertices
- Requires `points >= 2`

### `line(start, end)` / `polyline(points)`
- `path.is_closed() == false`

### `polygon(points)`
- `path.is_closed() == true`
- Requires `points.len() >= 3`

---

## Style

- A `Style` with `fill: None` and `stroke: None` produces no tessellation output
- `Style::filled(color)` sets a solid fill and no stroke
- `Style::stroked(stroke)` sets a stroke and no fill
- `Stroke::solid(color, width)` defaults to `LineCap::Butt` and `LineJoin::Miter`
- Gradient stops must be sorted by offset ascending; behaviour with unsorted stops is unspecified

---

## Scene Graph

- `Scene::new()` starts empty; `render` on an empty scene makes no driver calls
- `Scene::add(shape)` adds a node to the root group
- `Group::with_transform(t)` applies `t` to all children
- Transforms compose: a child's world transform = parent * child * shape
- `Scene::render` calls `driver.begin_frame()` before any draw calls
- `Scene::render` calls `driver.end_frame()` after all draw calls
- Each `Shape` in the scene produces at least one `upload_mesh` + `draw_mesh` call pair per enabled style component (fill / stroke)
