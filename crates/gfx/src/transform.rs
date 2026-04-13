use glam::{Mat3, Vec2};

/// 2D affine transform stored as a 2×3 column-major matrix.
/// Wraps `glam::Mat3` (homogeneous 3×3) with a 2D-focused API.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform(pub Mat3);

impl Transform {
    pub fn identity() -> Self {
        Self(Mat3::IDENTITY)
    }

    pub fn translate(x: f32, y: f32) -> Self {
        Self(Mat3::from_translation(Vec2::new(x, y)))
    }

    pub fn rotate(radians: f32) -> Self {
        Self(Mat3::from_angle(radians))
    }

    pub fn scale(sx: f32, sy: f32) -> Self {
        Self(Mat3::from_scale(Vec2::new(sx, sy)))
    }

    /// Compose: apply `self` first, then `other`.
    pub fn then(self, other: Transform) -> Self {
        Self(other.0 * self.0)
    }

    pub fn apply(&self, point: Vec2) -> Vec2 {
        self.0.transform_point2(point)
    }

    pub fn inverse(&self) -> Option<Transform> {
        // glam always returns a Mat3 from inverse(); singular matrices produce NaN.
        // Check the determinant to decide whether it's actually invertible.
        if self.0.determinant().abs() < 1e-6 {
            None
        } else {
            Some(Transform(self.0.inverse()))
        }
    }

    pub fn to_mat3(self) -> Mat3 {
        self.0
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::identity()
    }
}
