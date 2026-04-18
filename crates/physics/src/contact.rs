use glam::Vec2;

/// Result of a narrowphase collision test.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Contact {
    /// Unit vector from body A toward body B.
    pub normal: Vec2,
    /// Penetration depth; positive means the shapes are overlapping.
    pub depth: f32,
    /// Approximate world-space contact point.
    pub point: Vec2,
}
