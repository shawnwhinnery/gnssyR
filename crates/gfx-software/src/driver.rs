use gfx::driver::{GraphicsDriver, MeshHandle, Vertex};
use glam::Mat3;

// ---------------------------------------------------------------------------
// Mesh pool
// ---------------------------------------------------------------------------

struct StoredMesh {
    vertices: Vec<Vertex>,
    indices:  Vec<u32>,
}

struct DrawCall {
    handle:    MeshHandle,
    transform: Mat3,
    tint:      [f32; 4],
}

// ---------------------------------------------------------------------------
// Driver
// ---------------------------------------------------------------------------

/// Headless software driver backed by an in-memory pixel buffer.
///
/// Rasterises meshes CPU-side. Requires no GPU, no display, and no GPU
/// driver — suitable for automated tests and CI.
pub struct SoftwareDriver {
    width:      u32,
    height:     u32,
    pixels:     Vec<u32>,  // ARGB packed u32
    clear_color: u32,
    meshes:     Vec<StoredMesh>,
    draw_calls:  Vec<DrawCall>,
}

impl SoftwareDriver {
    /// Create a headless driver with the given dimensions.
    pub fn headless(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            pixels:      vec![0u32; (width * height) as usize],
            clear_color: 0,
            meshes:      Vec::new(),
            draw_calls:  Vec::new(),
        }
    }

    /// Read back the current framebuffer as ARGB pixels.
    pub fn pixels(&self) -> &[u32] {
        &self.pixels
    }

    pub fn width(&self)  -> u32 { self.width  }
    pub fn height(&self) -> u32 { self.height }
}

// ---------------------------------------------------------------------------
// Aspect-ratio projection
// ---------------------------------------------------------------------------

fn aspect_projection(width: u32, height: u32) -> glam::Mat3 {
    let w = width  as f32;
    let h = height as f32;
    if w > h {
        glam::Mat3::from_scale(glam::Vec2::new(h / w, 1.0))
    } else if h > w {
        glam::Mat3::from_scale(glam::Vec2::new(1.0, w / h))
    } else {
        glam::Mat3::IDENTITY
    }
}

impl GraphicsDriver for SoftwareDriver {
    fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.width  = width;
        self.height = height;
        self.pixels = vec![0u32; (width * height) as usize];
    }

    fn begin_frame(&mut self) {
        self.meshes.clear();
        self.draw_calls.clear();
    }

    fn end_frame(&mut self) {
        // Apply the clear colour first.
        self.pixels.fill(self.clear_color);

        // Rasterise each recorded draw call.
        let proj = aspect_projection(self.width, self.height);
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
    }

    fn present(&mut self) {}  // no-op in headless mode

    fn clear(&mut self, color: [f32; 4]) {
        self.clear_color = pack_argb(color);
    }

    fn upload_mesh(&mut self, vertices: &[Vertex], indices: &[u32]) -> MeshHandle {
        let handle = self.meshes.len() as MeshHandle;
        self.meshes.push(StoredMesh {
            vertices: vertices.to_vec(),
            indices:  indices.to_vec(),
        });
        handle
    }

    fn draw_mesh(&mut self, mesh: MeshHandle, transform: Mat3, color: [f32; 4]) {
        self.draw_calls.push(DrawCall {
            handle:    mesh,
            transform,
            tint:      color,
        });
    }

    fn backend_name(&self) -> &'static str { "CPU" }

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
