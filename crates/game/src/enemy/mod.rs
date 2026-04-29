pub mod dummy;

use glam::Vec2;
use physics::{BodyHandle, PhysicsWorld};

use crate::{camera::Camera, weapon::WeaponStats};

pub trait Enemy {
    fn body(&self) -> BodyHandle;
    fn health(&self) -> f32;
    fn is_alive(&self) -> bool {
        self.health() > 0.0
    }
    fn take_damage(&mut self, amount: f32);

    /// Advance AI one step.
    ///
    /// Returns `(origin, directions)` pairs — one entry per volley to spawn.
    /// Each pair maps directly to a set of projectiles the caller should add to the world.
    fn tick_ai(
        &mut self,
        dt: f32,
        player_positions: &[Vec2],
        physics: &mut PhysicsWorld,
    ) -> Vec<(Vec2, Vec<Vec2>)>;

    fn weapon_stats(&self) -> &WeaponStats;

    fn draw(
        &self,
        physics: &PhysicsWorld,
        driver: &mut dyn gfx::GraphicsDriver,
        camera: &Camera,
    );
}
