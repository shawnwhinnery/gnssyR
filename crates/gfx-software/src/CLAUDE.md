# CLAUDE.md — gfx-software/src/

## Purpose

CPU-based `GraphicsDriver`. Used for all headless tests and CI environments where no GPU is available. Provides a pixel-readable in-memory framebuffer.

## Files

| File | Responsibility |
|------|---------------|
| `lib.rs` | Crate root — exports `SoftwareDriver` |
| `driver.rs` | `SoftwareDriver`: implements `GraphicsDriver`; owns the framebuffer; `headless(w, h)` constructor; `pixels()` accessor |
| `raster.rs` | Software rasterizer: triangle scan-conversion using edge equations; fill and stroke rendering |

## Key API

```rust
let mut driver = SoftwareDriver::headless(512, 512);
driver.begin_frame();
// ... draw calls ...
driver.end_frame();
let pixels: &[u32] = driver.pixels();  // ARGB packed, row-major
```

## Invariants

- `present()` is a no-op — there is no window to swap to.
- `clear(color)` overwrites the entire framebuffer in one pass.
- Draw order is preserved — later draw calls appear on top of earlier ones.
- Raster output must be **deterministic** for the same input and frame sequence. Snapshot tests in `game/tests/` rely on this.
- Zero dependency on GPU, OS, or window system crates.
- Pixel format: packed ARGB `u32`, little-endian, row-major (top-left origin).
