use std::collections::HashMap;

use gilrs::{Event, EventType, Gilrs};

use crate::{
    backend::InputBackend,
    event::{Axis, Button, InputEvent},
    player::PlayerId,
};

/// Real-hardware gamepad backend powered by `gilrs`.
///
/// Maps up to 4 connected gamepads to [`PlayerId`]s in connection order.
/// Axis values below the 0.1 dead-zone are clamped to 0.0.
pub struct GilrsBackend {
    gilrs: Gilrs,
    player_map: HashMap<gilrs::GamepadId, PlayerId>,
    next_player: u8,
}

impl GilrsBackend {
    pub fn new() -> Self {
        let gilrs = Gilrs::new().expect("failed to initialise gilrs");

        // Register any gamepads that were already connected at startup.
        let mut player_map = HashMap::new();
        let mut next_player = 0u8;
        for (id, _pad) in gilrs.gamepads() {
            if next_player < 4 {
                player_map.insert(id, PlayerId(next_player));
                next_player += 1;
            }
        }

        Self {
            gilrs,
            player_map,
            next_player,
        }
    }
}

impl Default for GilrsBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl InputBackend for GilrsBackend {
    fn poll(&mut self) -> Vec<InputEvent> {
        let mut events = Vec::new();

        while let Some(Event { id, event, .. }) = self.gilrs.next_event() {
            match event {
                EventType::Connected => {
                    if !self.player_map.contains_key(&id) && self.next_player < 4 {
                        let pid = PlayerId(self.next_player);
                        self.player_map.insert(id, pid);
                        self.next_player += 1;
                        events.push(InputEvent::GamepadConnected(pid));
                    }
                }
                EventType::Disconnected => {
                    if let Some(pid) = self.player_map.remove(&id) {
                        events.push(InputEvent::GamepadDisconnected(pid));
                    }
                }
                EventType::ButtonPressed(btn, _) => {
                    if let Some(&pid) = self.player_map.get(&id) {
                        if let Some(button) = translate_button(btn) {
                            events.push(InputEvent::Button {
                                player: pid,
                                button,
                                pressed: true,
                            });
                        }
                    }
                }
                EventType::ButtonReleased(btn, _) => {
                    if let Some(&pid) = self.player_map.get(&id) {
                        if let Some(button) = translate_button(btn) {
                            events.push(InputEvent::Button {
                                player: pid,
                                button,
                                pressed: false,
                            });
                        }
                    }
                }
                EventType::AxisChanged(axis, raw, _) => {
                    if let Some(&pid) = self.player_map.get(&id) {
                        if let Some(ax) = translate_axis(axis) {
                            // Apply dead-zone: values inside ±0.1 become 0.
                            let value = if raw.abs() < 0.1 { 0.0 } else { raw };
                            events.push(InputEvent::Axis {
                                player: pid,
                                axis: ax,
                                value,
                            });
                        }
                    }
                }
                _ => {}
            }
        }

        events
    }
}

// ---------------------------------------------------------------------------
// Translation helpers
// ---------------------------------------------------------------------------

fn translate_button(btn: gilrs::Button) -> Option<Button> {
    use gilrs::Button as G;
    Some(match btn {
        G::South => Button::South,
        G::East => Button::East,
        G::West => Button::West,
        G::North => Button::North,
        G::LeftTrigger => Button::LeftBumper,
        G::RightTrigger => Button::RightBumper,
        G::LeftTrigger2 => Button::LeftTrigger,
        G::RightTrigger2 => Button::RightTrigger,
        G::LeftThumb => Button::LeftStick,
        G::RightThumb => Button::RightStick,
        G::DPadUp => Button::DPadUp,
        G::DPadDown => Button::DPadDown,
        G::DPadLeft => Button::DPadLeft,
        G::DPadRight => Button::DPadRight,
        G::Start => Button::Start,
        G::Select => Button::Select,
        _ => return None,
    })
}

fn translate_axis(axis: gilrs::Axis) -> Option<Axis> {
    use gilrs::Axis as G;
    Some(match axis {
        G::LeftStickX => Axis::LeftX,
        G::LeftStickY => Axis::LeftY,
        G::RightStickX => Axis::RightX,
        G::RightStickY => Axis::RightY,
        G::LeftZ => Axis::LeftTrigger,
        G::RightZ => Axis::RightTrigger,
        _ => return None,
    })
}
