use game::scenes::sandbox::SandboxScene;
use game::scenes::{Scene, SceneTransition};
use gfx_wgpu::WgpuDriver;
use window::App;

fn main() {
    #[cfg(feature = "gilrs")]
    let input_backend = input::GilrsBackend::new();
    #[cfg(not(feature = "gilrs"))]
    let input_backend = input::SimulatedBackend::new();

    let scene: Box<dyn Scene> = Box::new(SandboxScene::new());

    App::run(
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
        |scene, driver| {
            scene.draw(driver);
        },
    );
}
