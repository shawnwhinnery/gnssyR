use crate::world::World;

pub const GROUND_COLOR: [f32; 4] = [0.13, 0.14, 0.12, 1.0];

/// Sandbox scene for game development — top-down camera and typed-arena world.
pub fn draw_scene(driver: &mut dyn gfx::GraphicsDriver, world: &World) {
    world.draw(driver);
}
