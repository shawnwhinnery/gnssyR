use input::{InputEvent, event::{Button, KeyCode}};
use std::cell::Cell;

use super::{Scene, SceneTransition};
use crate::scenes::main_menu::MainMenuScene;

pub struct LevelSelectScene {
    back: Cell<bool>,
}

impl LevelSelectScene {
    pub fn new() -> Self {
        Self { back: Cell::new(false) }
    }
}

impl Default for LevelSelectScene {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene for LevelSelectScene {
    fn tick(&mut self, events: &[InputEvent]) -> Option<SceneTransition> {
        for event in events {
            if let InputEvent::Button { button: Button::Key(KeyCode::Escape), pressed: true, .. } = event {
                self.back.set(true);
            }
        }

        if self.back.get() {
            return Some(SceneTransition::Replace(Box::new(MainMenuScene::new())));
        }
        None
    }

    fn draw(&self, driver: &mut dyn gfx::GraphicsDriver) {
        let _ = driver;
    }

    fn draw_ui(&self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(egui::Color32::from_rgb(12, 12, 20)))
            .show(ctx, |ui| {
                let available = ui.available_size();
                let center = ui.clip_rect().center();
                let rect = egui::Rect::from_center_size(center, egui::vec2(320.0, available.y));
                ui.allocate_new_ui(egui::UiBuilder::new().max_rect(rect), |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(available.y * 0.25);

                        ui.label(
                            egui::RichText::new("Select Level")
                                .size(48.0)
                                .strong()
                                .color(egui::Color32::WHITE),
                        );

                        ui.add_space(48.0);

                        ui.label(
                            egui::RichText::new("Coming soon…")
                                .size(18.0)
                                .color(egui::Color32::from_rgb(160, 160, 180)),
                        );

                        ui.add_space(48.0);

                        if ui
                            .add_sized([200.0, 48.0], egui::Button::new(egui::RichText::new("Back").size(22.0)))
                            .clicked()
                        {
                            self.back.set(true);
                        }

                        ui.add_space(16.0);

                        ui.label(
                            egui::RichText::new("Esc to go back")
                                .size(13.0)
                                .color(egui::Color32::from_rgb(100, 100, 120)),
                        );
                    });
                });
            });
    }
}
