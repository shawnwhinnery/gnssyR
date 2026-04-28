use input::{
    InputEvent,
    event::{Button, KeyCode},
};
use std::cell::Cell;

use super::{Scene, SceneTransition};
use crate::scenes::level_select::LevelSelectScene;
use crate::scenes::sandbox::SandboxScene;

pub struct MainMenuScene {
    /// Set to true by "Start Game" to transition to level select.
    start_game: Cell<bool>,
    /// Set to true by "Start Sandbox" to jump directly to the sandbox.
    start_sandbox: Cell<bool>,
    /// Set to true by the Quit button.
    quit: Cell<bool>,
}

impl MainMenuScene {
    pub fn new() -> Self {
        Self {
            start_game: Cell::new(false),
            start_sandbox: Cell::new(false),
            quit: Cell::new(false),
        }
    }
}

impl Default for MainMenuScene {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene for MainMenuScene {
    fn tick(&mut self, events: &[InputEvent]) -> Option<SceneTransition> {
        for event in events {
            if let InputEvent::Button { button, pressed: true, .. } = event {
                match button {
                    Button::Key(KeyCode::Enter) | Button::Key(KeyCode::Space) => {
                        self.start_game.set(true);
                    }
                    Button::Key(KeyCode::Escape) => {
                        self.quit.set(true);
                    }
                    _ => {}
                }
            }
        }

        if self.start_game.get() {
            return Some(SceneTransition::Replace(Box::new(LevelSelectScene::new())));
        }
        if self.start_sandbox.get() {
            return Some(SceneTransition::Replace(Box::new(SandboxScene::new())));
        }
        if self.quit.get() {
            return Some(SceneTransition::Quit);
        }
        None
    }

    fn draw(&self, driver: &mut dyn gfx::GraphicsDriver) {
        let _ = driver;
    }

    fn draw_ui(&self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(
                egui::Frame::default()
                    .fill(egui::Color32::from_rgb(12, 12, 20)),
            )
            .show(ctx, |ui| {
                let available = ui.available_size();

                let center = ui.clip_rect().center();
                let rect = egui::Rect::from_center_size(
                    center,
                    egui::vec2(320.0, available.y),
                );
                ui.allocate_new_ui(egui::UiBuilder::new().max_rect(rect), |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(available.y * 0.25);

                        ui.label(
                            egui::RichText::new("gnssyR")
                                .size(64.0)
                                .strong()
                                .color(egui::Color32::WHITE),
                        );

                        ui.add_space(8.0);

                        ui.label(
                            egui::RichText::new("couch co-op · up to 4 players")
                                .size(16.0)
                                .color(egui::Color32::from_rgb(160, 160, 180)),
                        );

                        ui.add_space(48.0);

                        if ui
                            .add_sized(
                                [200.0, 48.0],
                                egui::Button::new(
                                    egui::RichText::new("Start Game").size(22.0),
                                ),
                            )
                            .clicked()
                        {
                            self.start_game.set(true);
                        }

                        ui.add_space(12.0);

                        if ui
                            .add_sized(
                                [200.0, 48.0],
                                egui::Button::new(
                                    egui::RichText::new("Start Sandbox").size(22.0),
                                ),
                            )
                            .clicked()
                        {
                            self.start_sandbox.set(true);
                        }

                        ui.add_space(12.0);

                        if ui
                            .add_sized(
                                [200.0, 48.0],
                                egui::Button::new(
                                    egui::RichText::new("Quit").size(22.0),
                                ),
                            )
                            .clicked()
                        {
                            self.quit.set(true);
                        }

                        ui.add_space(32.0);

                        ui.label(
                            egui::RichText::new("Enter / Space to start  ·  Esc to quit")
                                .size(13.0)
                                .color(egui::Color32::from_rgb(100, 100, 120)),
                        );
                    });
                });
            });
    }
}
