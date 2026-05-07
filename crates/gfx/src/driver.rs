use glam::Mat3;

/// Index into the driver's internal mesh pool.
pub type MeshHandle = u32;

/// Opaque id for a bitmap uploaded via [`GraphicsDriver::upload_texture`].
///
/// Unlike [`MeshHandle`], texture handles remain valid across `begin_frame` /
/// `end_frame` boundaries until explicitly freed with [`GraphicsDriver::free_texture`].
pub type TextureHandle = u64;

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

    /// Upload a bitmap in packed ARGB `u32` row-major order (`(a << 24) | (r << 16) | (g << 8) | b`).
    ///
    /// Returns a handle valid until [`GraphicsDriver::free_texture`] is called.
    /// `width` and `height` must be positive; if `pixels.len() != width * height`,
    /// behaviour is undefined.
    fn upload_texture(&mut self, pixels: &[u32], width: u32, height: u32) -> TextureHandle;

    /// Release resources for a texture uploaded with [`GraphicsDriver::upload_texture`].
    ///
    /// Freeing an unknown or already-freed handle is a no-op.
    fn free_texture(&mut self, handle: TextureHandle);

    /// Draw the full bitmap as a clip-space quad from `(-1, -1)` to `(1, 1)` with UVs mapping
    /// the image (row 0 at the top), transformed by `transform` (same space as
    /// [`GraphicsDriver::draw_mesh`]) and multiplied by `tint`.
    ///
    /// Invalid or freed texture handles are skipped (no draw, no panic).
    fn draw_bitmap(&mut self, texture: TextureHandle, transform: Mat3, tint: [f32; 4]);

    /// Notify the driver that the output surface has been resized.
    ///
    /// Must be called whenever the window dimensions change so that the driver
    /// can reconfigure its swapchain and update its aspect-ratio projection.
    fn resize(&mut self, width: u32, height: u32);

    /// Short identifier for the active rendering backend, shown in the HUD.
    fn backend_name(&self) -> &'static str;

    fn surface_size(&self) -> (u32, u32);
}
