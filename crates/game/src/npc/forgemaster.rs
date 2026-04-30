use glam::{Mat3, Vec2};
use physics::{Body, BodyHandle, Collider, PhysicsWorld};

use crate::camera::Camera;
use gfx::{
    shape::polygon,
    style::{Fill, LineCap, LineJoin, Stroke, Style},
    tessellate, Color,
};

use super::{FriendlyNpc, NpcKind};

pub const FORGEMASTER_RADIUS: f32 = 0.40;
const INTERACTION_RADIUS: f32 = 1.8;
const FILL_COLOR: u32 = 0xE8A020FF;
const STROKE_COLOR: u32 = 0x3A2000FF;

pub struct Forgemaster {
    pub body: BodyHandle,
}

impl Forgemaster {
    pub fn new(pos: Vec2, physics: &mut PhysicsWorld) -> Self {
        let body = physics.add_body(Body {
            position: pos,
            velocity: Vec2::ZERO,
            mass: f32::INFINITY,
            restitution: 0.0,
            collider: Collider::Circle { radius: FORGEMASTER_RADIUS },
        });
        Self { body }
    }
}

impl FriendlyNpc for Forgemaster {
    fn body(&self) -> BodyHandle {
        self.body
    }

    fn interaction_radius(&self) -> f32 {
        INTERACTION_RADIUS
    }

    fn kind(&self) -> NpcKind {
        NpcKind::Forgemaster
    }

    fn draw(&self, physics: &PhysicsWorld, driver: &mut dyn gfx::GraphicsDriver, camera: &Camera) {
        let pos = physics.body(self.body).position;
        let ndc = camera.world_to_ndc(pos);
        let r = camera.scale(FORGEMASTER_RADIUS * 1.25);

        // Regular hexagon (flat-top, 6 vertices).
        let verts: Vec<Vec2> = (0..6)
            .map(|i| {
                let angle = std::f32::consts::FRAC_PI_6 + std::f32::consts::FRAC_PI_3 * i as f32;
                ndc + Vec2::new(angle.cos(), angle.sin()) * r
            })
            .collect();

        let style = Style {
            fill: Some(Fill::Solid(Color::hex(FILL_COLOR))),
            stroke: Some(Stroke {
                color: Color::hex(STROKE_COLOR),
                width: 0.010,
                cap: LineCap::Round,
                join: LineJoin::Round,
            }),
        };

        for mesh in tessellate(&polygon(&verts), &style) {
            let handle = driver.upload_mesh(&mesh.vertices, &mesh.indices);
            driver.draw_mesh(handle, Mat3::IDENTITY, [1.0, 1.0, 1.0, 1.0]);
        }
    }
}
