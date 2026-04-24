/// Implemented by GPU-backed drivers that can render an egui frame.
///
/// [`App::run_with_ui`] calls [`EguiRenderer::prepare_egui`] once per frame
/// (after the game render closure returns) with the tessellated egui output.
/// The driver stores the data and renders it inside [`gfx::GraphicsDriver::end_frame`],
/// on top of game content, in the same command encoder.
pub trait EguiRenderer {
    fn prepare_egui(
        &mut self,
        primitives: Vec<egui::ClippedPrimitive>,
        textures_delta: egui::TexturesDelta,
        screen_size_px: [u32; 2],
        pixels_per_point: f32,
    );
}
