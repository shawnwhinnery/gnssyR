pub mod forgemaster;

use physics::{BodyHandle, PhysicsWorld};

use crate::camera::Camera;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NpcKind {
    Forgemaster,
}

pub trait FriendlyNpc {
    fn body(&self) -> BodyHandle;
    fn interaction_radius(&self) -> f32;
    fn kind(&self) -> NpcKind;
    fn draw(&self, physics: &PhysicsWorld, driver: &mut dyn gfx::GraphicsDriver, camera: &Camera);
}
