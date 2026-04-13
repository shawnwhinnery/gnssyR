# gfx-software

CPU-based `GraphicsDriver` implementation. No GPU or display required. Intended for headless testing and CI environments.

---

## Construction

- `SoftwareDriver::headless(width, height)` — allocates an in-memory pixel buffer
- No window handle required

---

## Behaviour

- Satisfies the full `GraphicsDriver` trait contract (defined in `gfx`)
- `clear` fills the pixel buffer with the packed ARGB value of `color`
- `upload_mesh` stores the mesh in a CPU-side pool
- `draw_mesh` rasterises triangles into the pixel buffer using barycentric interpolation
- `present` is a no-op in headless mode
- `pixels()` returns the current framebuffer as a `&[u32]` (ARGB packed) for assertion in tests

---

## Pixel Correctness

- A `clear` followed by no draw calls: all pixels equal the cleared color
- Drawing a filled triangle: all pixels within the triangle bounds reflect the triangle's color
- Transforms: a translated mesh renders at the expected pixel offset (within 1px tolerance)

---

## Headless Guarantee

- No GPU driver, display server, or OS window is required
- Must compile and run successfully in a Docker container with no GPU passthrough
