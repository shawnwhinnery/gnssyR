use game::scenes::main_menu::MainMenuScene;
use game::scenes::{Scene, SceneTransition};
use gfx_wgpu::WgpuDriver;
use window::App;

fn main() {
    #[cfg(feature = "gilrs")]
    let input_backend = input::GilrsBackend::new();
    #[cfg(not(feature = "gilrs"))]
    let input_backend = input::SimulatedBackend::new();

    let scene: Box<dyn Scene> = Box::new(MainMenuScene::new());
    let egui_ctx = egui::Context::default();
    let ctx_for_render = egui_ctx.clone();

    App::run_with_ui(
        scene,
        input_backend,
        |window| WgpuDriver::new(window),
        |scene, events| {
            if let Some(transition) = scene.tick(&events) {
                match transition {
                    SceneTransition::Replace(next) => *scene = next,
                    SceneTransition::Quit => { /* TODO: signal application exit */ }
                }
            }
        },
        move |scene, driver| {
            scene.draw(driver);
            scene.draw_ui(&ctx_for_render);
        },
        egui_ctx,
    );
}
