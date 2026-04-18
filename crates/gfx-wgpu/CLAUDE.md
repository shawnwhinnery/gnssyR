# CLAUDE.md — gfx-wgpu

## Scope

`gfx-wgpu` is the production GPU-backed `GraphicsDriver` implementation using `wgpu` and a window surface.

## Source of Truth

- Spec: `crates/gfx-wgpu/index.md`
- Tests plan: `crates/gfx-wgpu/test.md`
- Key modules: `crates/gfx-wgpu/src/driver.rs`, `crates/gfx-wgpu/src/buffer.rs`, `crates/gfx-wgpu/src/pipeline.rs`, shaders in `crates/gfx-wgpu/src/shaders/`

## Non-Negotiable Invariants

- Must satisfy the full `gfx::GraphicsDriver` contract.
- Driver creation depends on a valid window handle and compatible adapter/device.
- Uploaded mesh buffers are frame-scoped and recycled on subsequent frame boundaries.
- `present()` must submit command work and present the swapchain image.
- Surface resizing must be handled so `surface_size()` reflects current dimensions.

## Editing Guidance

- Keep behavior aligned with `gfx` contracts first, then optimize implementation details.
- Ensure shader/vertex layouts remain consistent with host-side buffer packing.
- Keep backend-specific errors understandable and fail fast on invalid device/surface states.
- Avoid introducing assumptions that break Vulkan/Metal/DX12 portability.

## Validation

- Run crate tests: `cargo test -p gfx-wgpu`
- If interfaces changed, run dependents: `cargo test -p window -p game`
