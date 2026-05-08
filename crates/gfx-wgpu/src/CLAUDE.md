# CLAUDE.md — gfx-wgpu/src/

## Purpose

Production GPU rendering via wgpu. Implements `GraphicsDriver` for the real game window plus `EguiRenderer` for in-game UI layered on top of game content.

## Files

| File | Responsibility |
|------|---------------|
| `lib.rs` | Crate root — exports `WgpuDriver` |
| `driver.rs` | `WgpuDriver`: implements `GraphicsDriver` + `EguiRenderer`; surface management, frame submission, resize, egui pass |
| `buffer.rs` | Per-frame vertex/index buffer allocation; GPU staging helpers |
| `pipeline.rs` | `RenderPipeline` setup for the fill and stroke WGSL shaders |
| `texture_store.rs` | `TextureStore`: wgpu texture allocation and lifetime management for `upload_texture` / `free_texture` |
| `textured_pipeline.rs` | Separate render pipeline for textured quad (`draw_bitmap`) |
| `shaders/` | WGSL shader source — see `shaders/CLAUDE.md` |

## Frame Flow (`driver.rs`)

```
begin_frame()     — acquire swapchain texture; begin command encoder
  upload_mesh()   — vertex/index buffer written to buffer.rs staging area
  draw_mesh()     — encodes draw call with pipeline.rs bind groups
  upload_texture() — texture_store.rs allocates wgpu texture + copies pixels
  draw_bitmap()   — textured_pipeline.rs draws quad
end_frame()
  egui pass       — egui_wgpu::Renderer draws UI on top (LoadOp::Load)
  submit encoder
present()         — swapchain present
```

## Invariants

- Mesh buffers are frame-scoped — allocated in `begin_frame`, invalid after next `begin_frame`.
- egui is always rendered last, on top of game content, using `LoadOp::Load` (never `Clear`).
- `surface_size()` returns current swapchain dimensions; always updated on resize before next frame.
- Shaders and host vertex layout must stay in sync — any `Vertex` struct change requires matching WGSL update.
- Keep portability across wgpu backends (Vulkan, Metal, DX12) — avoid backend-specific extensions.
- Never import `gfx-software` or game logic here.
