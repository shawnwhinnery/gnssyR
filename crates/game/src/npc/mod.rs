pub mod forgemaster;

use physics::{BodyHandle, PhysicsWorld};

use crate::{actor::ActorCore, camera::Camera};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NpcKind {
    Forgemaster,
}

pub trait FriendlyNpc {
    fn actor(&self) -> &ActorCore;

    fn body(&self) -> BodyHandle {
        self.actor().body
    }
    fn interaction_radius(&self) -> f32;
    fn kind(&self) -> NpcKind;
    fn draw(&self, physics: &PhysicsWorld, driver: &mut dyn gfx::GraphicsDriver, camera: &Camera);
}
