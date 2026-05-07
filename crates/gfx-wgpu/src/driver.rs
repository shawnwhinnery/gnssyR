use crate::buffer::MeshPool;
use crate::pipeline::FillPipeline;
use crate::texture_store::TextureStore;
use crate::textured_pipeline::{TexturedPipeline, TexturedVertex};
use gfx::aspect_projection;
use gfx::driver::{GraphicsDriver, MeshHandle, TextureHandle, Vertex};
use glam::Mat3;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use wgpu::util::DeviceExt;
use window::EguiRenderer;

// ---------------------------------------------------------------------------
// Per-draw uniform layout (must match fill.wgsl `Uniforms` struct)
//
// The Mat3 is stored as three vec4 columns (each column padded to 16 bytes)
// so the Rust and WGSL layouts agree without any manual padding tricks.
// ---------------------------------------------------------------------------

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    transform_col0: [f32; 4],
    transform_col1: [f32; 4],
    transform_col2: [f32; 4],
    tint: [f32; 4],
}

impl Uniforms {
    fn new(transform: Mat3, tint: [f32; 4]) -> Self {
        let c = transform.to_cols_array_2d(); // [[f32;3]; 3], column-major
        Self {
            transform_col0: [c[0][0], c[0][1], c[0][2], 0.0],
            transform_col1: [c[1][0], c[1][1], c[1][2], 0.0],
            transform_col2: [c[2][0], c[2][1], c[2][2], 0.0],
            tint,
        }
    }
}

// ---------------------------------------------------------------------------
// Recorded draw call
// ---------------------------------------------------------------------------

struct DrawCall {
    handle: MeshHandle,
    transform: Mat3,
    tint: [f32; 4],
}

struct TexturedDrawCall {
    handle: TextureHandle,
    transform: Mat3,
    tint: [f32; 4],
}

// ---------------------------------------------------------------------------
// Pending egui frame data (stored between prepare_egui and end_frame)
// ---------------------------------------------------------------------------

struct EguiData {
    primitives: Vec<egui::ClippedPrimitive>,
    textures_delta: egui::TexturesDelta,
    screen_size_px: [u32; 2],
    pixels_per_point: f32,
}

// ---------------------------------------------------------------------------
// Driver
// ---------------------------------------------------------------------------

/// Production GPU driver backed by wgpu (Vulkan / Metal / DX12).
pub struct WgpuDriver {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    pipeline: FillPipeline,
    mesh_pool: MeshPool,
    draw_calls: Vec<DrawCall>,
    textured_pipeline: TexturedPipeline,
    texture_store: TextureStore,
    textured_draw_calls: Vec<TexturedDrawCall>,
    bitmap_quad_vertex: wgpu::Buffer,
    bitmap_quad_index: wgpu::Buffer,
    clear_color: wgpu::Color,
    current_texture: Option<wgpu::SurfaceTexture>,
    egui_renderer: egui_wgpu::Renderer,
    pending_egui: Option<EguiData>,
}

impl WgpuDriver {
    /// Create a driver bound to `window`.
    ///
    /// Blocks the calling thread on GPU adapter/device initialisation.
    /// Panics if no compatible GPU adapter is found.
    ///
    /// # Safety
    /// The window handle must remain valid for the lifetime of this driver.
    /// `WinitApp` guarantees this by declaring `driver` before `window`.
    pub fn new(window: &(impl HasWindowHandle + HasDisplayHandle)) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // SAFETY: window outlives driver (see doc comment).
        let surface = unsafe {
            instance
                .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
                    raw_display_handle: window
                        .display_handle()
                        .expect("no display handle")
                        .as_raw(),
                    raw_window_handle: window.window_handle().expect("no window handle").as_raw(),
                })
                .expect("failed to create wgpu surface")
        };

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .expect("no compatible GPU adapter found");

        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default(), None))
                .expect("failed to acquire GPU device");

        let caps = surface.get_capabilities(&adapter);
        let format = caps.formats[0];

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: 800,
            height: 600,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let pipeline = FillPipeline::new(&device, format);
        let textured_pipeline = TexturedPipeline::new(&device, format);
        let texture_store = TextureStore::new(&device);
        let egui_renderer = egui_wgpu::Renderer::new(&device, format, None, 1, false);

        let quad_verts = [
            TexturedVertex {
                position: [-1.0, -1.0],
                uv: [0.0, 1.0],
            },
            TexturedVertex {
                position: [1.0, -1.0],
                uv: [1.0, 1.0],
            },
            TexturedVertex {
                position: [1.0, 1.0],
                uv: [1.0, 0.0],
            },
            TexturedVertex {
                position: [-1.0, 1.0],
                uv: [0.0, 0.0],
            },
        ];
        let quad_idx: [u32; 6] = [0, 1, 2, 0, 2, 3];
        let bitmap_quad_vertex = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("bitmap_quad_verts"),
            contents: bytemuck::cast_slice(&quad_verts),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let bitmap_quad_index = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("bitmap_quad_idx"),
            contents: bytemuck::cast_slice(&quad_idx),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            device,
            queue,
            surface,
            config,
            pipeline,
            mesh_pool: MeshPool::default(),
            draw_calls: Vec::new(),
            textured_pipeline,
            texture_store,
            textured_draw_calls: Vec::new(),
            bitmap_quad_vertex,
            bitmap_quad_index,
            clear_color: wgpu::Color::BLACK,
            current_texture: None,
            egui_renderer,
            pending_egui: None,
        }
    }
}

impl GraphicsDriver for WgpuDriver {
    fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
    }

    fn begin_frame(&mut self) {
        // Recycle all mesh buffers from the previous frame.
        self.mesh_pool.clear();
        self.draw_calls.clear();
        self.textured_draw_calls.clear();
        self.pending_egui = None;

        match self.surface.get_current_texture() {
            Ok(texture) => self.current_texture = Some(texture),
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                self.surface.configure(&self.device, &self.config);
            }
            Err(e) => eprintln!("surface error in begin_frame: {e}"),
        }
    }

    fn clear(&mut self, color: [f32; 4]) {
        self.clear_color = wgpu::Color {
            r: color[0] as f64,
            g: color[1] as f64,
            b: color[2] as f64,
            a: color[3] as f64,
        };
    }

    fn upload_mesh(&mut self, vertices: &[Vertex], indices: &[u32]) -> MeshHandle {
        self.mesh_pool.upload(&self.device, vertices, indices)
    }

    fn draw_mesh(&mut self, mesh: MeshHandle, transform: Mat3, color: [f32; 4]) {
        self.draw_calls.push(DrawCall {
            handle: mesh,
            transform,
            tint: color,
        });
    }

    fn upload_texture(&mut self, pixels: &[u32], width: u32, height: u32) -> TextureHandle {
        self.texture_store
            .upload(&self.device, &self.queue, pixels, width, height)
    }

    fn free_texture(&mut self, handle: TextureHandle) {
        self.texture_store.free(handle);
    }

    fn draw_bitmap(&mut self, texture: TextureHandle, transform: Mat3, tint: [f32; 4]) {
        self.textured_draw_calls.push(TexturedDrawCall {
            handle: texture,
            transform,
            tint,
        });
    }

    fn end_frame(&mut self) {
        let Some(texture) = &self.current_texture else {
            return;
        };

        let view = texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("frame"),
            });

        // Build per-draw bind groups before entering the render pass so that
        // the uniform buffers' lifetimes don't need to outlast the pass block.
        let proj = aspect_projection(self.config.width, self.config.height);
        let bind_groups: Vec<wgpu::BindGroup> = self
            .draw_calls
            .iter()
            .map(|draw| {
                let uniforms = Uniforms::new(proj * draw.transform, draw.tint);
                let buf = self
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("draw_uniforms"),
                        contents: bytemuck::bytes_of(&uniforms),
                        usage: wgpu::BufferUsages::UNIFORM,
                    });
                self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("draw_bind_group"),
                    layout: &self.pipeline.bind_group_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: buf.as_entire_binding(),
                    }],
                })
            })
            .collect();

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("fill_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            pass.set_pipeline(&self.pipeline.pipeline);

            for (draw, bind_group) in self.draw_calls.iter().zip(bind_groups.iter()) {
                let Some(mesh) = self.mesh_pool.get(draw.handle) else {
                    continue;
                };
                pass.set_bind_group(0, bind_group, &[]);
                pass.set_vertex_buffer(0, mesh.vertex_buf.slice(..));
                pass.set_index_buffer(mesh.index_buf.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..mesh.index_count, 0, 0..1);
            }
        }

        // Textured bitmap pass (after vector fills, before egui).
        if !self.textured_draw_calls.is_empty() {
            let textured_bind_groups: Vec<wgpu::BindGroup> = {
                let mut v = Vec::new();
                for draw in &self.textured_draw_calls {
                    let Some(stored) = self.texture_store.get(draw.handle) else {
                        continue;
                    };
                    let uniforms = Uniforms::new(proj * draw.transform, draw.tint);
                    let buf = self
                        .device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("textured_draw_uniforms"),
                            contents: bytemuck::bytes_of(&uniforms),
                            usage: wgpu::BufferUsages::UNIFORM,
                        });
                    let bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("textured_draw_bind"),
                        layout: &self.textured_pipeline.bind_group_layout,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: buf.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: wgpu::BindingResource::TextureView(&stored.view),
                            },
                            wgpu::BindGroupEntry {
                                binding: 2,
                                resource: wgpu::BindingResource::Sampler(
                                    self.texture_store.sampler(),
                                ),
                            },
                        ],
                    });
                    v.push(bg);
                }
                v
            };

            {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("bitmap_pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

                pass.set_pipeline(&self.textured_pipeline.pipeline);

                for bind_group in textured_bind_groups.iter() {
                    pass.set_bind_group(0, bind_group, &[]);
                    pass.set_vertex_buffer(0, self.bitmap_quad_vertex.slice(..));
                    pass.set_index_buffer(
                        self.bitmap_quad_index.slice(..),
                        wgpu::IndexFormat::Uint32,
                    );
                    pass.draw_indexed(0..6, 0, 0..1);
                }
            }
        }

        // Render egui on top of game content (same encoder, Load op preserves pixels).
        if let Some(egui) = self.pending_egui.take() {
            let screen_descriptor = egui_wgpu::ScreenDescriptor {
                size_in_pixels: egui.screen_size_px,
                pixels_per_point: egui.pixels_per_point,
            };

            for (id, image_delta) in &egui.textures_delta.set {
                self.egui_renderer
                    .update_texture(&self.device, &self.queue, *id, image_delta);
            }
            self.egui_renderer.update_buffers(
                &self.device,
                &self.queue,
                &mut encoder,
                &egui.primitives,
                &screen_descriptor,
            );

            {
                let egui_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("egui_pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
                // egui_wgpu::Renderer::render requires RenderPass<'static>
                let mut egui_pass = egui_pass.forget_lifetime();
                self.egui_renderer
                    .render(&mut egui_pass, &egui.primitives, &screen_descriptor);
            }

            for id in &egui.textures_delta.free {
                self.egui_renderer.free_texture(id);
            }
        }

        self.queue.submit([encoder.finish()]);
    }

    fn present(&mut self) {
        if let Some(texture) = self.current_texture.take() {
            texture.present();
        }
    }

    fn backend_name(&self) -> &'static str {
        "GPU"
    }

    fn surface_size(&self) -> (u32, u32) {
        (self.config.width, self.config.height)
    }
}

// ---------------------------------------------------------------------------
// EguiRenderer impl — stores tessellated UI data for end_frame to consume
// ---------------------------------------------------------------------------

impl EguiRenderer for WgpuDriver {
    fn prepare_egui(
        &mut self,
        primitives: Vec<egui::ClippedPrimitive>,
        textures_delta: egui::TexturesDelta,
        screen_size_px: [u32; 2],
        pixels_per_point: f32,
    ) {
        self.pending_egui = Some(EguiData {
            primitives,
            textures_delta,
            screen_size_px,
            pixels_per_point,
        });
    }
}
