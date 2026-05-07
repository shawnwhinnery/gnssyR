use gfx::aspect_projection;
use gfx::driver::{GraphicsDriver, MeshHandle, TextureHandle, Vertex};
use glam::Mat3;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Mesh pool
// ---------------------------------------------------------------------------

struct StoredMesh {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}

struct DrawCall {
    handle: MeshHandle,
    transform: Mat3,
    tint: [f32; 4],
}

struct SoftwareTexture {
    pixels: Vec<u32>,
    width: u32,
    height: u32,
}

struct TexturedDrawCall {
    handle: TextureHandle,
    transform: Mat3,
    tint: [f32; 4],
}

// ---------------------------------------------------------------------------
// Driver
// ---------------------------------------------------------------------------

/// Headless software driver backed by an in-memory pixel buffer.
///
/// Rasterises meshes CPU-side. Requires no GPU, no display, and no GPU
/// driver — suitable for automated tests and CI.
pub struct SoftwareDriver {
    width: u32,
    height: u32,
    pixels: Vec<u32>, // ARGB packed u32
    clear_color: u32,
    meshes: Vec<StoredMesh>,
    draw_calls: Vec<DrawCall>,
    textures: HashMap<TextureHandle, SoftwareTexture>,
    next_texture_id: TextureHandle,
    textured_draw_calls: Vec<TexturedDrawCall>,
}

impl SoftwareDriver {
    /// Create a headless driver with the given dimensions.
    pub fn headless(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            pixels: vec![0u32; (width * height) as usize],
            clear_color: 0,
            meshes: Vec::new(),
            draw_calls: Vec::new(),
            textures: HashMap::new(),
            next_texture_id: 1,
            textured_draw_calls: Vec::new(),
        }
    }

    /// Read back the current framebuffer as ARGB pixels.
    pub fn pixels(&self) -> &[u32] {
        &self.pixels
    }

    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }
}

impl GraphicsDriver for SoftwareDriver {
    fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.width = width;
        self.height = height;
        self.pixels = vec![0u32; (width * height) as usize];
    }

    fn begin_frame(&mut self) {
        self.meshes.clear();
        self.draw_calls.clear();
        self.textured_draw_calls.clear();
    }

    fn end_frame(&mut self) {
        // Apply the clear colour first.
        self.pixels.fill(self.clear_color);

        let proj = aspect_projection(self.width, self.height);

        // Vector meshes (replace pixels under triangles).
        for draw in &self.draw_calls {
            if let Some(mesh) = self.meshes.get(draw.handle as usize) {
                crate::raster::rasterize(
                    &mut self.pixels,
                    self.width,
                    self.height,
                    &mesh.vertices,
                    &mesh.indices,
                    proj * draw.transform,
                    draw.tint,
                );
            }
        }

        // Bitmaps (alpha-composite over existing pixels).
        for draw in &self.textured_draw_calls {
            let Some(tex) = self.textures.get(&draw.handle) else {
                continue;
            };
            crate::raster::raster_bitmap(
                &mut self.pixels,
                self.width,
                self.height,
                &tex.pixels,
                tex.width,
                tex.height,
                proj * draw.transform,
                draw.tint,
            );
        }
    }

    fn present(&mut self) {} // no-op in headless mode

    fn clear(&mut self, color: [f32; 4]) {
        self.clear_color = pack_argb(color);
    }

    fn upload_mesh(&mut self, vertices: &[Vertex], indices: &[u32]) -> MeshHandle {
        let handle = self.meshes.len() as MeshHandle;
        self.meshes.push(StoredMesh {
            vertices: vertices.to_vec(),
            indices: indices.to_vec(),
        });
        handle
    }

    fn draw_mesh(&mut self, mesh: MeshHandle, transform: Mat3, color: [f32; 4]) {
        self.draw_calls.push(DrawCall {
            handle: mesh,
            transform,
            tint: color,
        });
    }

    fn upload_texture(&mut self, pixels: &[u32], width: u32, height: u32) -> TextureHandle {
        if width == 0 || height == 0 || pixels.len() != (width * height) as usize {
            return 0;
        }
        let id = self.next_texture_id;
        self.next_texture_id = self.next_texture_id.saturating_add(1).max(1);
        self.textures.insert(
            id,
            SoftwareTexture {
                pixels: pixels.to_vec(),
                width,
                height,
            },
        );
        id
    }

    fn free_texture(&mut self, handle: TextureHandle) {
        self.textures.remove(&handle);
    }

    fn draw_bitmap(&mut self, texture: TextureHandle, transform: Mat3, tint: [f32; 4]) {
        self.textured_draw_calls.push(TexturedDrawCall {
            handle: texture,
            transform,
            tint,
        });
    }

    fn backend_name(&self) -> &'static str {
        "CPU"
    }

    fn surface_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

fn pack_argb(color: [f32; 4]) -> u32 {
    let r = (color[0] * 255.0) as u32;
    let g = (color[1] * 255.0) as u32;
    let b = (color[2] * 255.0) as u32;
    let a = (color[3] * 255.0) as u32;
    (a << 24) | (r << 16) | (g << 8) | b
}
