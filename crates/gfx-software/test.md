# gfx-software Crate Tests

The software driver is headless (no GPU, no display) and exposes `pixels()` for
direct framebuffer inspection. This makes it the primary correctness oracle for
the entire rendering pipeline.

All pixel values are packed ARGB `u32`: `(a << 24) | (r << 16) | (g << 8) | b`.

---

## Construction

### `headless_construction`
`SoftwareDriver::headless(200, 100)` does not panic.

### `surface_size_matches_construction`
`SoftwareDriver::headless(200, 100).surface_size()` == `(200, 100)`.

### `initial_pixel_count`
`pixels().len()` == `width * height` (200 × 100 = 20 000).

---

## Frame Lifecycle

### `frame_lifecycle_no_panic`
Calling `begin_frame()`, `end_frame()`, `present()` in order does not panic.

### `present_is_noop`
Calling `present()` does not change `pixels()`.

### `multiple_frames_no_panic`
Running 10 complete frames (`begin_frame` → `end_frame` → `present`) does not panic.

---

## Clear

### `clear_fills_all_pixels`
After `clear([1.0, 0.0, 0.0, 1.0])` (solid red), every pixel in `pixels()` equals
`0xFF_FF_00_00`.

### `clear_black`
After `clear([0.0, 0.0, 0.0, 1.0])`, every pixel equals `0xFF_00_00_00`.

### `clear_white`
After `clear([1.0, 1.0, 1.0, 1.0])`, every pixel equals `0xFF_FF_FF_FF`.

### `clear_transparent`
After `clear([0.0, 0.0, 0.0, 0.0])`, every pixel equals `0x00_00_00_00`.

### `multiple_clears_last_wins`
`clear(red)` then `clear(blue)` in the same frame: all pixels == `0xFF_00_00_FF`.

### `clear_does_not_affect_next_frame_after_begin`
Clear to red. Call `begin_frame()`. Clear to blue. Call `end_frame()`.
All pixels == blue (the red clear is not visible).

---

## Upload Mesh

### `upload_mesh_returns_handle_zero`
The first call to `upload_mesh` returns `0`.

### `upload_mesh_second_returns_one`
Two `upload_mesh` calls return `0` then `1`.

### `upload_empty_mesh_is_valid`
`upload_mesh(&[], &[])` returns a valid handle (no panic).

### `handles_recycled_after_begin_frame`
Upload a mesh in frame 1 (handle 0). Call `begin_frame()` to start frame 2.
Upload again — the new handle is `0` again (pool was cleared).

---

## Draw Mesh — Triangle Rasterisation

All raster tests use a 100×100 headless driver. Clip space runs −1…+1 on both
axes. Pixel coordinates are derived as:

```
pixel_x = (clip_x + 1.0) / 2.0 * width        (rounds to nearest)
pixel_y = (1.0 - clip_y) / 2.0 * height        (y-axis is flipped)
```

The pixel index for (px, py) is `py * width + px`.

### `draw_triangle_centroid_pixel`
- Driver: 100×100
- Clear to black.
- Triangle vertices (all white `[1,1,1,1]`):
  - `(0.0,  0.6)` → approx pixel (50, 20)
  - `(-0.6, -0.6)` → approx pixel (20, 80)
  - `( 0.6, -0.6)` → approx pixel (80, 80)
- Transform: `Mat3::IDENTITY`, tint: `[1,1,1,1]`
- After `end_frame`: pixel at the centroid (50, 60) is white (`0xFF_FF_FF_FF`).
- Pixel at (5, 5) (outside triangle) remains black (`0xFF_00_00_00`).

### `draw_triangle_vertex_colours_interpolated`
- Driver: 100×100, clear to black.
- Triangle:
  - `(0.0, 0.8)` — red   `[1,0,0,1]`
  - `(-0.8, -0.8)` — green `[0,1,0,1]`
  - `(0.8, -0.8)` — blue  `[0,0,1,1]`
- The pixel nearest each vertex tip should be close to that vertex's pure colour
  (each channel ≥ 0.8, others ≤ 0.2 after unpacking).

### `draw_fills_triangle_interior_not_exterior`
- Simple triangle covering the upper-right quadrant of the screen.
  Vertices: `(0.0, 0.0)`, `(1.0, 0.0)`, `(0.0, 1.0)`, all white.
- Pixel at (75, 25) (inside) is white.
- Pixel at (10, 90) (outside, lower-left) is black.

### `draw_triangle_with_translation`
- Driver: 100×100, clear to black.
- Upload the same triangle centred at clip origin (vertices roughly ±0.3).
- Call `draw_mesh` twice:
  1. Transform = `Mat3::IDENTITY` (centred at clip origin → pixel 50, 50)
  2. Transform = `Mat3::from_translation(Vec2::new(0.5, 0.0))` (shifted right 0.5 clip units → pixel ~75, 50)
- After `end_frame`:
  - Centroid of the first instance (pixel 50, 50) is white.
  - Centroid of the second instance (pixel 75, 50) is white.
  - They are separate draws (the gap between them is black).

### `draw_triangle_with_tint`
- Upload a white triangle `[1,1,1,1]`.
- `draw_mesh(handle, Mat3::IDENTITY, [1.0, 0.0, 0.0, 1.0])` (red tint).
- Centroid pixel is red (`0xFF_FF_00_00`).

### `clear_and_draw_background_preserved`
- Clear to green.
- Draw a small triangle in the centre (white).
- Pixels far from the triangle remain green.
- Pixels inside the triangle are white.

### `draw_multiple_meshes_in_one_frame`
- Upload two separate triangles that do not overlap.
- Draw both in the same frame.
- After `end_frame`: centroid of each triangle has the expected colour.

### `draw_filled_rect_all_interior_pixels`
Construct a rectangle mesh (two triangles) covering clip `(-0.5, -0.5)` to `(0.5, 0.5)`.
After drawing, every pixel strictly inside the rectangle bounds is the fill colour.
**Tolerance**: up to 1 pixel on each edge is allowed to be background.

---

## Headless Guarantee

### `compiles_without_display`
The crate compiles and all tests above run in a process with no `DISPLAY`,
`WAYLAND_DISPLAY`, or GPU passthrough (verified by CI environment).

### `no_gpu_driver_required`
`SoftwareDriver::headless` constructs without any GPU driver calls
(verified by `strace`-level: no `/dev/dri` open in test process).
*This is a documentation test — verified by the absence of GPU deps in Cargo.toml.*
