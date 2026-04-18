use crate::Aabb;
use glam::Vec2;

/// Collision shape in local (body) space.
///
/// Vertices are expected in CCW winding order. Mesh triangles follow the same convention.
#[derive(Clone, Debug)]
pub enum Collider {
    /// Cheapest shape — O(1) circle–circle test.
    Circle { radius: f32 },

    /// Convex polygon. SAT-based narrowphase.
    Convex { vertices: Vec<Vec2> },

    /// Arbitrary triangle mesh (e.g. static level geometry).
    /// Broadphase filters to AABB per triangle before SAT.
    Mesh {
        vertices: Vec<Vec2>,
        indices: Vec<[u32; 3]>,
    },
}

impl Collider {
    /// Tight axis-aligned bounding box in local space.
    pub fn local_aabb(&self) -> Aabb {
        match self {
            Collider::Circle { radius } => Aabb {
                min: Vec2::splat(-radius),
                max: Vec2::splat(*radius),
            },
            Collider::Convex { vertices } => Aabb::from_points(vertices),
            Collider::Mesh { vertices, .. } => Aabb::from_points(vertices),
        }
    }
}
