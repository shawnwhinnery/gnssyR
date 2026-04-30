use glam::{Mat3, Vec2};
use physics::{BodyHandle, PhysicsWorld};

use crate::camera::Camera;
use gfx::{tessellate, Path, Style};

/// Shared physics + orientation state for every in-world character.
#[derive(Debug, Clone, Copy)]
pub struct ActorCore {
    pub body: BodyHandle,
    pub facing: Vec2,
}

impl ActorCore {
    pub fn new(body: BodyHandle) -> Self {
        Self {
            body,
            facing: Vec2::X,
        }
    }
}

/// Common interface for players, enemies, and NPCs.
pub trait Actor {
    fn actor(&self) -> &ActorCore;

    fn draw(&self, physics: &PhysicsWorld, driver: &mut dyn gfx::GraphicsDriver, camera: &Camera);
}

pub fn draw_shape(
    driver: &mut dyn gfx::GraphicsDriver,
    path: &Path,
    style: &Style,
    transform: Mat3,
) {
    for mesh in tessellate(path, style) {
        let handle = driver.upload_mesh(&mesh.vertices, &mesh.indices);
        driver.draw_mesh(handle, transform, [1.0, 1.0, 1.0, 1.0]);
    }
}
