use glam::Vec2;
use input::{
    event::{Axis, Button, InputEvent},
    player::PlayerId,
};

#[derive(Default, Clone, Copy)]
pub struct PlayerIntent {
    pub move_dir: Vec2,
    pub aim_dir: Vec2,
    /// True when `aim_dir` was set by an analog stick rather than the cursor.
    pub aim_from_stick: bool,
    pub fire: bool,
}

pub struct InputSnapshot([PlayerIntent; 4]);

impl InputSnapshot {
    pub fn from_events(events: &[InputEvent], cursor_ndc: Vec2) -> Self {
        let mut dpad = [[false; 4]; 4];
        let mut left_sticks = [Vec2::ZERO; 4];
        let mut right_sticks = [Vec2::ZERO; 4];
        let mut fire = [false; 4];

        for event in events {
            match event {
                InputEvent::Button {
                    player,
                    button,
                    pressed,
                } => {
                    let idx = slot_index(*player);
                    match button {
                        Button::DPadUp => dpad[idx][0] = *pressed,
                        Button::DPadDown => dpad[idx][1] = *pressed,
                        Button::DPadLeft => dpad[idx][2] = *pressed,
                        Button::DPadRight => dpad[idx][3] = *pressed,
                        Button::South if *pressed => fire[idx] = true,
                        _ => {}
                    }
                }
                InputEvent::Axis {
                    player,
                    axis,
                    value,
                } => {
                    let idx = slot_index(*player);
                    match axis {
                        Axis::LeftX => left_sticks[idx].x = *value,
                        Axis::LeftY => left_sticks[idx].y = -*value,
                        Axis::RightX => right_sticks[idx].x = *value,
                        Axis::RightY => right_sticks[idx].y = -*value,
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        let mut players = [PlayerIntent::default(); 4];
        for idx in 0..4 {
            let mut move_dir = Vec2::ZERO;
            let stick = left_sticks[idx];
            if stick.length_squared() > 0.01 {
                move_dir = stick;
            } else {
                if dpad[idx][0] {
                    move_dir.y += 1.0;
                }
                if dpad[idx][1] {
                    move_dir.y -= 1.0;
                }
                if dpad[idx][2] {
                    move_dir.x -= 1.0;
                }
                if dpad[idx][3] {
                    move_dir.x += 1.0;
                }
            }
            if move_dir.length_squared() > 1.0 {
                move_dir = move_dir.normalize();
            }

            let mut aim_dir = if idx == 0 { cursor_ndc } else { Vec2::X };
            if aim_dir.length_squared() > 1e-6 {
                aim_dir = aim_dir.normalize();
            } else {
                aim_dir = Vec2::X;
            }

            let aim_from_stick = right_sticks[idx].length_squared() > 0.01;
            if aim_from_stick {
                aim_dir = right_sticks[idx].normalize();
            }

            players[idx] = PlayerIntent {
                move_dir,
                aim_dir,
                aim_from_stick,
                fire: fire[idx],
            };
        }

        Self(players)
    }

    pub fn player(&self, idx: usize) -> PlayerIntent {
        self.0[idx]
    }
}

#[derive(Default)]
pub struct InputState {
    dpad: [[bool; 4]; 4],
    left_sticks: [Vec2; 4],
    right_sticks: [Vec2; 4],
    fire: [bool; 4],
    cursor_ndc: Vec2,
}

impl InputState {
    pub fn apply_events(&mut self, events: &[InputEvent], cursor_ndc: Vec2) {
        self.cursor_ndc = cursor_ndc;
        for event in events {
            match event {
                InputEvent::Button {
                    player,
                    button,
                    pressed,
                } => {
                    let idx = slot_index(*player);
                    match button {
                        Button::DPadUp => self.dpad[idx][0] = *pressed,
                        Button::DPadDown => self.dpad[idx][1] = *pressed,
                        Button::DPadLeft => self.dpad[idx][2] = *pressed,
                        Button::DPadRight => self.dpad[idx][3] = *pressed,
                        Button::South => self.fire[idx] = *pressed,
                        _ => {}
                    }
                }
                InputEvent::Axis {
                    player,
                    axis,
                    value,
                } => {
                    let idx = slot_index(*player);
                    match axis {
                        Axis::LeftX => self.left_sticks[idx].x = *value,
                        Axis::LeftY => self.left_sticks[idx].y = -*value,
                        Axis::RightX => self.right_sticks[idx].x = *value,
                        Axis::RightY => self.right_sticks[idx].y = -*value,
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

    pub fn snapshot(&self) -> InputSnapshot {
        let mut players = [PlayerIntent::default(); 4];
        for idx in 0..4 {
            let mut move_dir = Vec2::ZERO;
            let stick = self.left_sticks[idx];
            if stick.length_squared() > 0.01 {
                move_dir = stick;
            } else {
                if self.dpad[idx][0] {
                    move_dir.y += 1.0;
                }
                if self.dpad[idx][1] {
                    move_dir.y -= 1.0;
                }
                if self.dpad[idx][2] {
                    move_dir.x -= 1.0;
                }
                if self.dpad[idx][3] {
                    move_dir.x += 1.0;
                }
            }
            if move_dir.length_squared() > 1.0 {
                move_dir = move_dir.normalize();
            }

            let mut aim_dir = if idx == 0 { self.cursor_ndc } else { Vec2::X };
            if aim_dir.length_squared() > 1e-6 {
                aim_dir = aim_dir.normalize();
            } else {
                aim_dir = Vec2::X;
            }

            let aim_from_stick = self.right_sticks[idx].length_squared() > 0.01;
            if aim_from_stick {
                aim_dir = self.right_sticks[idx].normalize();
            }

            players[idx] = PlayerIntent {
                move_dir,
                aim_dir,
                aim_from_stick,
                fire: self.fire[idx],
            };
        }
        InputSnapshot(players)
    }
}

impl Default for InputSnapshot {
    fn default() -> Self {
        Self([PlayerIntent::default(); 4])
    }
}

fn slot_index(player: PlayerId) -> usize {
    usize::from(player.0).min(3)
}

#[cfg(test)]
mod tests {
    use super::*;
    use input::player::PlayerId;

    #[test]
    fn dpad_maps_to_move_dir() {
        let events = vec![InputEvent::Button {
            player: PlayerId::P1,
            button: Button::DPadUp,
            pressed: true,
        }];
        let snapshot = InputSnapshot::from_events(&events, Vec2::ZERO);
        assert_eq!(snapshot.player(0).move_dir, Vec2::Y);
    }

    #[test]
    fn stick_overrides_dpad() {
        let events = vec![
            InputEvent::Button {
                player: PlayerId::P1,
                button: Button::DPadLeft,
                pressed: true,
            },
            InputEvent::Axis {
                player: PlayerId::P1,
                axis: Axis::LeftX,
                value: 1.0,
            },
        ];
        let snapshot = InputSnapshot::from_events(&events, Vec2::ZERO);
        assert_eq!(snapshot.player(0).move_dir, Vec2::X);
    }

    #[test]
    fn cursor_maps_to_aim() {
        let snapshot = InputSnapshot::from_events(&[], Vec2::new(0.0, 1.0));
        assert_eq!(snapshot.player(0).aim_dir, Vec2::Y);
    }

    #[test]
    fn gamepad_maps_to_correct_slot() {
        let events = vec![InputEvent::Axis {
            player: PlayerId::P2,
            axis: Axis::LeftX,
            value: 1.0,
        }];
        let snapshot = InputSnapshot::from_events(&events, Vec2::ZERO);
        assert_eq!(snapshot.player(0).move_dir, Vec2::ZERO);
        assert_eq!(snapshot.player(1).move_dir, Vec2::X);
    }
}
