use glam::Vec2;

/// Axis-aligned bounding box used for broadphase filtering.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Aabb {
    pub min: Vec2,
    pub max: Vec2,
}

impl Aabb {
    /// Tight box around a non-empty point set.
    pub fn from_points(pts: &[Vec2]) -> Self {
        let mut min = Vec2::splat(f32::MAX);
        let mut max = Vec2::splat(f32::NEG_INFINITY);
        for &p in pts {
            min = min.min(p);
            max = max.max(p);
        }
        Aabb { min, max }
    }

    /// Shift the box by `offset`.
    pub fn translate(self, offset: Vec2) -> Self {
        Aabb {
            min: self.min + offset,
            max: self.max + offset,
        }
    }

    /// True iff the boxes share interior (touching edges do not count).
    pub fn overlaps(self, other: Self) -> bool {
        self.min.x < other.max.x
            && self.max.x > other.min.x
            && self.min.y < other.max.y
            && self.max.y > other.min.y
    }

    /// Grow min/max outward by `amount` on all sides.
    pub fn expand(self, amount: f32) -> Self {
        let d = Vec2::splat(amount);
        Aabb {
            min: self.min - d,
            max: self.max + d,
        }
    }
}
