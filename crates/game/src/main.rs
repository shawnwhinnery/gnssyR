use game::{GameMode, PauseState, World};
use gfx_wgpu::WgpuDriver;
use window::App;

struct GameState {
    world: World,
    pause: PauseState,
}

fn main() {
    // Use the real gamepad backend when the `gilrs` feature is enabled,
    // otherwise fall back to the no-op simulated backend (keyboard still works
    // via winit events regardless of which input backend is chosen here).
    #[cfg(feature = "gilrs")]
    let input_backend = input::GilrsBackend::new();
    #[cfg(not(feature = "gilrs"))]
    let input_backend = input::SimulatedBackend::new();

    App::run(
        GameState { world: World::new(), pause: PauseState::new() },
        input_backend,
        |window| WgpuDriver::new(window),
        |state, events| {
            // PauseState owns the mode transition; update it first so the
            // correct mode is visible to all downstream systems this frame.
            state.pause.tick(&events);

            match state.pause.mode() {
                GameMode::Playing => {
                    state.world.tick(events);
                }
                GameMode::Paused => {
                    // Simulation is frozen. Future: route events to menu
                    // navigation here, passing `GameMode::Paused` to any
                    // system that needs to behave differently in this mode.
                }
            }
        },
        |state, driver| {
            // App has already called begin_frame() for us.
            game::sandbox::draw_scene(driver, &state.world);
            state.pause.draw(driver);
        },
    );
}
