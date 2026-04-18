use glam::Mat3;

/// Index into the driver's internal mesh pool.
pub type MeshHandle = u32;

/// A single GPU-ready vertex.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
}

/// CPU-side triangle mesh produced by tessellation.
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

/// Swappable rendering backend.
///
/// Implementations: `WgpuDriver` (production), `SoftwareDriver` (headless tests).
/// The vector layer tessellates paths into `Mesh`es and submits them here —
/// the driver has no knowledge of paths, styles, or the scene graph.
pub trait GraphicsDriver {
    fn begin_frame(&mut self);
    fn end_frame(&mut self);
    fn present(&mut self);

    fn clear(&mut self, color: [f32; 4]);

    /// Upload a mesh to the driver and return a handle.
    fn upload_mesh(&mut self, vertices: &[Vertex], indices: &[u32]) -> MeshHandle;

    /// Draw a previously uploaded mesh with an affine transform and tint.
    fn draw_mesh(&mut self, mesh: MeshHandle, transform: Mat3, color: [f32; 4]);

    /// Notify the driver that the output surface has been resized.
    ///
    /// Must be called whenever the window dimensions change so that the driver
    /// can reconfigure its swapchain and update its aspect-ratio projection.
    fn resize(&mut self, width: u32, height: u32);

    /// Short identifier for the active rendering backend, shown in the HUD.
    fn backend_name(&self) -> &'static str;

    fn surface_size(&self) -> (u32, u32);
}
