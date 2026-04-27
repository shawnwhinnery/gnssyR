use glam::Vec2;

pub const HALF_VIEW: f32 = 5.0;

pub struct Camera {
    pub half_view: f32,
    pub position: Vec2,
    velocity: Vec2,
    pub smooth_time: f32,
}

impl Camera {
    pub fn world_to_ndc(&self, p: Vec2) -> Vec2 {
        (p - self.position) / self.half_view
    }

    pub fn ndc_to_world(&self, p: Vec2) -> Vec2 {
        p * self.half_view + self.position
    }

    pub fn scale(&self, world_len: f32) -> f32 {
        world_len / self.half_view
    }

    pub fn update(&mut self, target: Vec2, dt: f32) {
        let omega = 2.0 / self.smooth_time;
        let x = omega * dt;
        let decay = 1.0 / (1.0 + x + 0.48 * x * x + 0.235 * x * x * x);
        let delta = self.position - target;
        let temp = (self.velocity + omega * delta) * dt;
        self.velocity = (self.velocity - omega * temp) * decay;
        self.position = target + (delta + temp) * decay;
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            half_view: HALF_VIEW,
            position: Vec2::ZERO,
            velocity: Vec2::ZERO,
            smooth_time: 0.3,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn camera_starts_at_origin() {
        assert_eq!(Camera::default().position, Vec2::ZERO);
    }

    #[test]
    fn camera_moves_toward_target() {
        let mut cam = Camera::default();
        cam.position = Vec2::new(10.0, 0.0);
        let target = Vec2::ZERO;
        let start_dist = cam.position.distance(target);
        for _ in 0..10 {
            cam.update(target, 1.0 / 60.0);
        }
        assert!(cam.position.distance(target) < start_dist);
    }

    #[test]
    fn camera_reaches_target() {
        let mut cam = Camera::default();
        cam.position = Vec2::new(10.0, 5.0);
        let target = Vec2::ZERO;
        // Simulate 3 seconds at 60 Hz
        for _ in 0..180 {
            cam.update(target, 1.0 / 60.0);
        }
        assert!(cam.position.distance(target) < 0.01);
    }

    #[test]
    fn camera_no_overshoot() {
        let mut cam = Camera::default();
        cam.position = Vec2::new(10.0, 0.0);
        let target = Vec2::ZERO;
        for _ in 0..300 {
            cam.update(target, 1.0 / 60.0);
            assert!(cam.position.x >= -0.001, "overshot: x = {}", cam.position.x);
        }
    }
}
