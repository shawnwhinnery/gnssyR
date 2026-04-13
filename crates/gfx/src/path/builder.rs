use glam::Vec2;
use super::{Path, Segment};

/// Fluent builder for constructing a [`Path`].
#[derive(Debug, Default)]
pub struct PathBuilder {
    segments: Vec<Segment>,
}

impl PathBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn move_to(mut self, p: Vec2) -> Self {
        self.segments.push(Segment::Move(p));
        self
    }

    pub fn line_to(mut self, p: Vec2) -> Self {
        self.segments.push(Segment::Line(p));
        self
    }

    pub fn quad_to(mut self, cp: Vec2, end: Vec2) -> Self {
        self.segments.push(Segment::Quad { cp, end });
        self
    }

    pub fn cubic_to(mut self, cp1: Vec2, cp2: Vec2, end: Vec2) -> Self {
        self.segments.push(Segment::Cubic { cp1, cp2, end });
        self
    }

    pub fn arc_to(mut self, center: Vec2, radius: f32, start_angle: f32, end_angle: f32) -> Self {
        self.segments.push(Segment::Arc { center, radius, start_angle, end_angle });
        self
    }

    pub fn close(self) -> Path {
        Path { segments: self.segments, closed: true }
    }

    pub fn build(self) -> Path {
        Path { segments: self.segments, closed: false }
    }
}
