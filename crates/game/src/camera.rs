use glam::Vec2;

pub const HALF_VIEW: f32 = 5.0;

pub struct Camera {
    pub half_view: f32,
}

impl Camera {
    pub fn world_to_ndc(&self, p: Vec2) -> Vec2 {
        p / self.half_view
    }

    pub fn ndc_to_world(&self, p: Vec2) -> Vec2 {
        p * self.half_view
    }

    pub fn scale(&self, world_len: f32) -> f32 {
        world_len / self.half_view
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            half_view: HALF_VIEW,
        }
    }
}
