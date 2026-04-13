use glam::Vec2;
use input::event::{Axis, Button, InputEvent};

// World units from centre to edge (must match sandbox.rs).
const HALF_VIEW:     f32 = 5.0;
const PLAYER_RADIUS: f32 = 0.5;
const PLAYER_SPEED:  f32 = 6.0; // world units per second
const MOVE_BOUND:    f32 = HALF_VIEW - PLAYER_RADIUS;

/// Top-level game state.
pub struct GameState {
    start:      std::time::Instant,
    last_tick:  std::time::Instant,
    fps:        f32,

    // Player
    player_pos: Vec2,
    move_up:    bool,
    move_down:  bool,
    move_left:  bool,
    move_right: bool,
    stick_x:    f32,
    stick_y:    f32,
}

impl GameState {
    pub fn new() -> Self {
        let now = std::time::Instant::now();
        Self {
            start:      now,
            last_tick:  now,
            fps:        0.0,
            player_pos: Vec2::ZERO,
            move_up:    false,
            move_down:  false,
            move_left:  false,
            move_right: false,
            stick_x:    0.0,
            stick_y:    0.0,
        }
    }

    /// Call once per frame — processes input events and integrates movement.
    pub fn tick(&mut self, events: Vec<InputEvent>) {
        let now = std::time::Instant::now();
        let dt  = now.duration_since(self.last_tick).as_secs_f32();
        self.last_tick = now;
        self.fps = self.fps * 0.9 + (1.0 / dt.max(1e-6)) * 0.1;

        // Update held-button and axis state from events.
        for event in events {
            match event {
                InputEvent::Button { button: Button::DPadUp,    pressed, .. } => self.move_up    = pressed,
                InputEvent::Button { button: Button::DPadDown,  pressed, .. } => self.move_down  = pressed,
                InputEvent::Button { button: Button::DPadLeft,  pressed, .. } => self.move_left  = pressed,
                InputEvent::Button { button: Button::DPadRight, pressed, .. } => self.move_right = pressed,
                InputEvent::Axis   { axis: Axis::LeftX, value, .. } => self.stick_x =  value,
                // Gamepad Y is inverted (up = negative); negate so +Y = up in world space.
                InputEvent::Axis   { axis: Axis::LeftY, value, .. } => self.stick_y = -value,
                _ => {}
            }
        }

        // Build velocity vector from whichever input is active.
        let stick = Vec2::new(self.stick_x, self.stick_y);
        let mut vel = if stick.length_squared() > 0.01 {
            // Analog stick wins when pushed past dead-zone.
            stick
        } else {
            let mut v = Vec2::ZERO;
            if self.move_up    { v.y += 1.0; }
            if self.move_down  { v.y -= 1.0; }
            if self.move_left  { v.x -= 1.0; }
            if self.move_right { v.x += 1.0; }
            v
        };

        // Normalise so diagonal isn't faster than cardinal.
        if vel.length_squared() > 1.0 { vel = vel.normalize(); }

        self.player_pos += vel * PLAYER_SPEED * dt;
        self.player_pos  = self.player_pos.clamp(
            Vec2::splat(-MOVE_BOUND),
            Vec2::splat( MOVE_BOUND),
        );
    }

    /// Seconds elapsed since the game started.
    pub fn elapsed_secs(&self) -> f32 {
        self.start.elapsed().as_secs_f32()
    }

    /// Smoothed frames-per-second.
    pub fn fps(&self) -> f32 { self.fps }

    /// Current player position in world units.
    pub fn player_pos(&self) -> Vec2 { self.player_pos }
}

impl Default for GameState {
    fn default() -> Self { Self::new() }
}
