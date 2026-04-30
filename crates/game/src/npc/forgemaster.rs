use glam::{Mat3, Vec2};
use physics::{Body, Collider, PhysicsWorld};

use crate::{
    actor::{draw_shape, Actor, ActorCore},
    camera::Camera,
};
use gfx::{
    shape::polygon,
    style::{Fill, LineCap, LineJoin, Stroke, Style},
    Color,
};

use super::{FriendlyNpc, NpcKind};

pub const FORGEMASTER_RADIUS: f32 = 0.40;
const INTERACTION_RADIUS: f32 = 1.8;
const FILL_COLOR: u32 = 0xE8A020FF;
const STROKE_COLOR: u32 = 0x3A2000FF;

pub struct Forgemaster {
    pub actor: ActorCore,
}

impl Forgemaster {
    pub fn new(pos: Vec2, physics: &mut PhysicsWorld) -> Self {
        let body = physics.add_body(Body {
            position: pos,
            velocity: Vec2::ZERO,
            mass: f32::INFINITY,
            restitution: 0.0,
            collider: Collider::Circle {
                radius: FORGEMASTER_RADIUS,
            },
        });
        Self {
            actor: ActorCore::new(body),
        }
    }
}

impl Actor for Forgemaster {
    fn actor(&self) -> &ActorCore {
        &self.actor
    }

    fn draw(&self, physics: &PhysicsWorld, driver: &mut dyn gfx::GraphicsDriver, camera: &Camera) {
        let pos = physics.body(self.actor.body).position;
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

        draw_shape(driver, &polygon(&verts), &style, Mat3::IDENTITY);
    }
}

impl FriendlyNpc for Forgemaster {
    fn actor(&self) -> &ActorCore {
        &self.actor
    }

    fn interaction_radius(&self) -> f32 {
        INTERACTION_RADIUS
    }

    fn kind(&self) -> NpcKind {
        NpcKind::Forgemaster
    }

    fn draw(&self, physics: &PhysicsWorld, driver: &mut dyn gfx::GraphicsDriver, camera: &Camera) {
        <Self as Actor>::draw(self, physics, driver, camera);
    }
}
