# gfx-wgpu Crate Tests

`WgpuDriver` requires a real GPU and window. Tests here use two strategies:

1. **Smoke tests** — create a headless wgpu instance (no window) to test the
   non-rendering parts of the driver (construction, handle management, call
   ordering). These run in CI if a Vulkan software renderer (lavapipe) is present.

2. **Pixel-readback tests** — render to an offscreen texture and read back pixels
   via `wgpu::Buffer` copy. These verify that the GPU pipeline produces the same
   output as the software driver for basic geometry.

All smoke tests are marked `#[ignore]` by default and require
`cargo test -- --include-ignored` or a CI environment with GPU/lavapipe.

---

## Construction

### `new_from_headless_surface`
Construct a `WgpuDriver` using an offscreen wgpu surface (no OS window).
Must not panic and must return a driver with `surface_size()` returning
non-zero dimensions.
**Requires:** Vulkan adapter (real GPU or lavapipe).

---

## Frame Lifecycle

### `frame_lifecycle_no_panic`
A complete frame — `begin_frame()`, `end_frame()`, `present()` — does not panic.

### `clear_does_not_panic`
`clear([0.2, 0.4, 0.6, 1.0])` between `begin_frame` and `end_frame` does not panic.

### `multiple_frames_no_panic`
Running 5 complete frames does not panic.

---

## Mesh Upload

### `upload_mesh_returns_handle_zero`
The first `upload_mesh` in a frame returns `MeshHandle` `0`.

### `upload_second_mesh_returns_one`
Two `upload_mesh` calls in the same frame return `0` then `1`.

### `upload_empty_mesh_no_panic`
`upload_mesh(&[], &[])` returns a valid handle without panicking.

### `handles_recycled_after_begin_frame`
Upload a mesh (handle 0) in frame 1. Call `begin_frame()` to start frame 2.
`upload_mesh` returns `0` again (pool was cleared).

---

## Draw Mesh

### `draw_mesh_no_panic`
`draw_mesh(handle, Mat3::IDENTITY, [1.0, 1.0, 1.0, 1.0])` with a valid handle
does not panic.

### `draw_mesh_multiple_calls_no_panic`
10 consecutive `draw_mesh` calls with the same handle in one frame do not panic.

### `draw_mesh_with_translation_no_panic`
`draw_mesh(handle, Mat3::from_translation(Vec2::new(0.5, 0.0)), [1,1,1,1])` does
not panic.

---

## Pixel Readback (GPU correctness)

These tests render to an offscreen `wgpu::Texture`, copy to a `wgpu::Buffer`,
and map it for CPU readback. They verify that the wgpu pipeline produces
visually correct output matching the software driver's results.

### `clear_color_matches_readback`
Render a frame with `clear([1.0, 0.0, 0.0, 1.0])` and no draws.
Every pixel in the readback buffer is red.

### `triangle_centroid_is_white`
Same geometry as `gfx-software::draw_triangle_centroid_pixel` above.
The centroid pixel in the GPU readback is white (within a tolerance of ±2 per
channel to account for GPU rounding).

### `tint_applied_to_white_mesh`
Upload a white mesh. Draw with red tint `[1,0,0,1]`.
Centroid pixel readback is red.

### `two_draws_same_handle_both_visible`
Draw the same mesh twice with different transforms (non-overlapping).
Both instance centroids are visible in the readback.

---

## Platform

### `surface_size_non_zero`
`surface_size()` returns `(w, h)` with both `w > 0` and `h > 0`.

### `surface_size_reflects_config`
After `resize(640, 480)`, `surface_size()` == `(640, 480)`.

---

## Bitmap

### `upload_texture_draw_bitmap_no_panic`
With a valid GPU driver: `upload_texture`, `begin_frame`, `clear`, `draw_bitmap`, `end_frame`,
`present`, `free_texture` completes without panic.

### `bitmap_readback_matches_expected` (optional)
Offscreen readback of a solid texture matches expected RGBA within ±2 per channel
(same tolerance as mesh readback tests).

