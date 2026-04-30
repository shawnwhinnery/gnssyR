use crate::player::PlayerId;

/// Unified input event emitted by any [`InputBackend`].
///
/// All hardware differences (gamepad vs keyboard/mouse) are normalised here.
#[derive(Debug, Clone)]
pub enum InputEvent {
    /// A digital button was pressed or released.
    Button {
        player: PlayerId,
        button: Button,
        pressed: bool,
    },
    /// An analogue axis changed value (−1.0 … +1.0).
    Axis {
        player: PlayerId,
        axis: Axis,
        value: f32,
    },
    /// Mouse cursor moved (screen-space delta).
    MouseMove { dx: f32, dy: f32 },
    /// Mouse cursor absolute position in NDC ([-1.0, 1.0], Y-up, origin at window centre).
    CursorMoved { x: f32, y: f32 },
    /// A gamepad connected.
    GamepadConnected(PlayerId),
    /// A gamepad disconnected.
    GamepadDisconnected(PlayerId),
}

/// Abstract digital buttons shared across gamepad and keyboard mappings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Button {
    // Face
    South,
    East,
    West,
    North,
    // Shoulders
    LeftBumper,
    RightBumper,
    // Triggers (digital threshold)
    LeftTrigger,
    RightTrigger,
    // Thumbsticks (click)
    LeftStick,
    RightStick,
    // D-pad
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
    // Menu
    Start,
    Select,
    // Keyboard fallback
    Key(KeyCode),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Axis {
    LeftX,
    LeftY,
    RightX,
    RightY,
    LeftTrigger,
    RightTrigger,
}

/// Minimal key codes for keyboard input fallback.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyCode {
    W,
    A,
    S,
    D,
    Up,
    Down,
    Left,
    Right,
    Space,
    Enter,
    Escape,
    E,
    // Extend as needed
}
