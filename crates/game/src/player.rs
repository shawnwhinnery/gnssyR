use glam::{Mat3, Vec2};
use physics::{Body, Collider, PhysicsWorld};

use crate::{
    actor::{draw_shape, Actor, ActorCore},
    camera::Camera,
    weapon::{Weapon, WeaponStats},
};
use gfx::{
    shape::{circle, line},
    style::{Fill, LineCap, LineJoin, Stroke, Style},
    Color,
};

pub const PLAYER_RADIUS: f32 = 0.5;
pub const PLAYER_SPEED: f32 = 6.0;
pub const PLAYER_COLORS: [u32; 4] = [0x2979FFFF, 0xFF5252FF, 0x66BB6AFF, 0xFFEB3BFF];

pub struct Player {
    pub slot: usize,
    pub actor: ActorCore,
    pub health: f32,
    pub color: Color,
    pub weapon: Weapon,
}

impl Player {
    pub fn new(slot: usize, start_pos: Vec2, physics: &mut PhysicsWorld) -> Self {
        let body = physics.add_body(Body {
            position: start_pos,
            velocity: Vec2::ZERO,
            mass: 1.0,
            restitution: 0.3,
            collider: Collider::Circle {
                radius: PLAYER_RADIUS,
            },
        });
        Self {
            slot,
            actor: ActorCore::new(body),
            health: 100.0,
            color: Color::hex(PLAYER_COLORS[slot.min(PLAYER_COLORS.len() - 1)]),
            weapon: Weapon::new(WeaponStats::default()),
        }
    }
}

impl Actor for Player {
    fn actor(&self) -> &ActorCore {
        &self.actor
    }

    fn draw(&self, physics: &PhysicsWorld, driver: &mut dyn gfx::GraphicsDriver, camera: &Camera) {
        if self.health <= 0.0 {
            return;
        }
        let pos = physics.body(self.actor.body).position;
        let ndc_pos = camera.world_to_ndc(pos);
        let aim_end = camera.world_to_ndc(pos + self.actor.facing * 2.0);

        draw_shape(
            driver,
            &line(ndc_pos, aim_end),
            &Style::stroked(Stroke {
                color: Color::hex(0xFFFFFFCC),
                width: 0.006,
                cap: LineCap::Round,
                join: LineJoin::Round,
            }),
            Mat3::IDENTITY,
        );

        draw_shape(
            driver,
            &circle(ndc_pos, camera.scale(PLAYER_RADIUS)),
            &Style {
                fill: Some(Fill::Solid(self.color)),
                stroke: Some(Stroke {
                    color: Color::hex(0x000000FF),
                    width: 0.008,
                    cap: LineCap::Round,
                    join: LineJoin::Round,
                }),
            },
            Mat3::IDENTITY,
        );
    }
}
