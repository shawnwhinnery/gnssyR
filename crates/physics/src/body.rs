use crate::Collider;
use glam::Vec2;

/// Opaque index into a [`PhysicsWorld`]. Cheaply copyable.
///
/// Using a handle after calling `remove_body` panics.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BodyHandle(pub usize);

/// A rigid body tracked by the physics world.
#[derive(Clone, Debug)]
pub struct Body {
    /// World-space position.
    pub position: Vec2,
    /// World-space velocity (units per second).
    pub velocity: Vec2,
    /// Mass in arbitrary units. `f32::INFINITY` makes the body static.
    pub mass: f32,
    /// Bounciness coefficient in `[0.0, 1.0]`.
    pub restitution: f32,
    /// Collision shape in local space.
    pub collider: Collider,
}

impl Body {
    /// True when the body has infinite mass and will not be moved by impulses.
    pub fn is_static(&self) -> bool {
        self.mass.is_infinite()
    }
}
