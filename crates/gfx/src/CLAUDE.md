# CLAUDE.md — gfx/src/

## Purpose

The backend-agnostic graphics contract. Defines shared types and the `GraphicsDriver` trait. Nothing here knows about wgpu, pixels, or OS windows.

## Files

| File | Responsibility |
|------|---------------|
| `lib.rs` | Crate root — re-exports all public types |
| `driver.rs` | `GraphicsDriver` trait; `MeshHandle`, `TextureHandle` opaque types; full frame lifecycle contract |
| `color.rs` | `Color` (RGBA f32); constructors (`from_rgb`, `from_hex`), named constants, alpha ops |
| `transform.rs` | `Transform` (affine 2D matrix); `identity`, `translate`, `rotate`, `scale`, `then`, `apply` |
| `style.rs` | `Style` (rendering intent); `Fill` (solid / gradient), `Stroke` (width + color) |
| `shape.rs` | Shape constructors → `Path`: `circle`, `rect`, `polygon`, `line`, `arc` |
| `scene.rs` | `Scene` graph: `SceneNode` tree; `scene.render(driver)` flattens transforms and issues draw calls |
| `view.rs` | View/projection helpers for NDC-space draw-call positioning |
| `path/` | Path type, builder, parametric evaluation, tessellation — see `path/CLAUDE.md` |

## Frame Lifecycle Contract (`driver.rs`)

```
driver.begin_frame()
  driver.clear(color)
  handle = driver.upload_mesh(vertices, indices)
  driver.draw_mesh(handle, transform, style)
  tex    = driver.upload_texture(width, height, pixels)
  driver.draw_bitmap(tex, transform)
  driver.free_texture(tex)
driver.end_frame()
driver.present()
```

**Mesh handles are frame-scoped** — they become invalid after the next `begin_frame`. Never cache them across frames.

## Invariants

- No backend crates (`wgpu`, `pixels`, `winit`) may be imported here.
- Transform composition: `a.then(b).apply(p) == b.apply(a.apply(p))` — do not alter this semantics.
- `surface_size() -> (u32, u32)` must reflect the current render target at the moment it is called.
- Undefined behavior in the spec (e.g. drawing after `end_frame`) must remain explicitly undefined — do not silently add guarantees.
