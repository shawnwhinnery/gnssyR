use glam::{Mat3, Vec2};
use physics::{Body, BodyHandle, Collider, PhysicsWorld};

use crate::{
    camera::Camera,
    weapon::{Weapon, WeaponStats},
};
use gfx::{
    shape::polygon,
    style::{Fill, LineCap, LineJoin, Stroke, Style},
    tessellate, Color,
};

use super::Enemy;

pub const DUMMY_RADIUS: f32 = 0.45;
const DUMMY_SPEED: f32 = 2.5;
const FIRE_RANGE: f32 = 8.0;
const DUMMY_COLOR: u32 = 0xCC2222FF;

pub struct Dummy {
    pub body: BodyHandle,
    pub facing: Vec2,
    health: f32,
    weapon: Weapon,
}

impl Dummy {
    pub fn new(pos: Vec2, physics: &mut PhysicsWorld) -> Self {
        let body = physics.add_body(Body {
            position: pos,
            velocity: Vec2::ZERO,
            mass: 1.0,
            restitution: 0.2,
            collider: Collider::Circle { radius: DUMMY_RADIUS },
        });
        Self {
            body,
            facing: Vec2::X,
            health: 50.0,
            weapon: Weapon::new(WeaponStats {
                fire_rate: 1.5,
                projectiles_per_shot: 1,
                shot_arc: 0.0,
                burst_count: 1,
                burst_delay: 0.05,
                jitter: 0.08,
                projectile_speed: 8.0,
                projectile_size: 0.10,
                projectile_lifetime: 3.0,
                piercing: 0,
                damage_total: 15.0,
                recoil_force: 0.0,
            }),
        }
    }
}

impl Enemy for Dummy {
    fn body(&self) -> BodyHandle {
        self.body
    }

    fn health(&self) -> f32 {
        self.health
    }

    fn take_damage(&mut self, amount: f32) {
        self.health = (self.health - amount).max(0.0);
    }

    fn weapon_stats(&self) -> &WeaponStats {
        &self.weapon.stats
    }

    fn tick_ai(
        &mut self,
        dt: f32,
        player_positions: &[Vec2],
        physics: &mut PhysicsWorld,
    ) -> Vec<(Vec2, Vec<Vec2>)> {
        let my_pos = physics.body(self.body).position;

        let closest = player_positions
            .iter()
            .copied()
            .min_by(|a, b| {
                a.distance(my_pos)
                    .partial_cmp(&b.distance(my_pos))
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

        let Some(target) = closest else {
            physics.body_mut(self.body).velocity = Vec2::ZERO;
            return vec![];
        };

        let to_target = target - my_pos;
        let dist = to_target.length();

        if dist > 1e-4 {
            self.facing = to_target / dist;
        }
        physics.body_mut(self.body).velocity = self.facing * DUMMY_SPEED;

        let fire_intent = dist < FIRE_RANGE;
        let volleys = self.weapon.tick(dt, fire_intent);

        if volleys > 0 {
            let dirs = self.weapon.volley_directions(self.facing);
            vec![(my_pos, dirs)]
        } else {
            vec![]
        }
    }

    fn draw(
        &self,
        physics: &PhysicsWorld,
        driver: &mut dyn gfx::GraphicsDriver,
        camera: &Camera,
    ) {
        if self.health <= 0.0 {
            return;
        }

        let pos = physics.body(self.body).position;
        let ndc = camera.world_to_ndc(pos);
        let r = camera.scale(DUMMY_RADIUS * 1.3);

        // Equilateral triangle pointing in `facing` direction.
        let forward = self.facing;
        let right = Vec2::new(forward.y, -forward.x);

        let tip = ndc + forward * r;
        let base_l = ndc - forward * (r * 0.5) + right * (r * 0.866);
        let base_r = ndc - forward * (r * 0.5) - right * (r * 0.866);

        let verts = [tip, base_l, base_r];
        let style = Style {
            fill: Some(Fill::Solid(Color::hex(DUMMY_COLOR))),
            stroke: Some(Stroke {
                color: Color::hex(0x000000FF),
                width: 0.008,
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
