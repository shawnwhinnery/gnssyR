# CLAUDE.md — gfx-wgpu/src/shaders/

## Purpose

WGSL GPU shader programs. One per render pipeline. Keep these as generic geometry pipelines — no game-specific logic belongs here.

## Files

| File | Pipeline | Role |
|------|----------|------|
| `fill.wgsl` | Fill pipeline (`pipeline.rs`) | Solid-color triangle rasterization; interpolates per-vertex color |
| `stroke.wgsl` | Stroke pipeline (`pipeline.rs`) | Outline / stroke rasterization |
| `textured.wgsl` | Textured pipeline (`textured_pipeline.rs`) | Textured quad rendering for `draw_bitmap` |

## Invariants

- **Vertex layout must match the host** — the `Vertex` struct in `buffer.rs` and the `@location` bindings in WGSL must be byte-for-byte compatible. Any change to one requires a change to the other.
- **Uniforms**: transform matrix and viewport size are passed as push constants or uniform buffers; these must be bound before each draw call that uses them.
- **Portability**: shaders must compile and run correctly on Vulkan, Metal, and DX12 (wgpu's target backends). Do not use backend-specific extensions or non-portable features.
- **No game logic**: shaders transform vertices and sample textures — they know nothing about game entities, camera, or world coordinates.
