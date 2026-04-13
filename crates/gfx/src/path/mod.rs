pub mod builder;
pub mod parametric;
pub(crate) mod tessellate;

pub use builder::PathBuilder;

use glam::Vec2;

/// A sequence of path segments, open or closed.
#[derive(Debug, Clone)]
pub struct Path {
    pub(crate) segments: Vec<Segment>,
    pub(crate) closed:   bool,
}

#[derive(Debug, Clone)]
pub(crate) enum Segment {
    Move(Vec2),
    Line(Vec2),
    Quad { cp: Vec2, end: Vec2 },
    Cubic { cp1: Vec2, cp2: Vec2, end: Vec2 },
    Arc { center: Vec2, radius: f32, start_angle: f32, end_angle: f32 },
}

impl Path {
    pub fn builder() -> PathBuilder {
        PathBuilder::new()
    }

    pub fn is_closed(&self) -> bool {
        self.closed
    }

    pub fn segment_count(&self) -> usize {
        self.segments.len()
    }
}
