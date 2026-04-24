mod world;

use input::InputEvent;

use crate::pause::PauseState;
use world::World;

use super::{Scene, SceneTransition};

pub struct SandboxScene {
    world: World,
    pause: PauseState,
}

impl SandboxScene {
    pub fn new() -> Self {
        Self { world: World::new(), pause: PauseState::new() }
    }
}

impl Default for SandboxScene {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene for SandboxScene {
    fn tick(&mut self, events: &[InputEvent]) -> Option<SceneTransition> {
        self.pause.tick(events);
        if !self.pause.is_paused() {
            self.world.tick(events);
        }
        None
    }

    fn draw(&self, driver: &mut dyn gfx::GraphicsDriver) {
        self.world.draw(driver);
    }

    fn draw_ui(&self, ctx: &egui::Context) {
        self.pause.draw_ui(ctx);
    }
}
