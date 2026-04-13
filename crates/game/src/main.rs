mod state;

use gfx_wgpu::WgpuDriver;
use state::GameState;
use window::App;

fn main() {
    // Use the real gamepad backend when the `gilrs` feature is enabled,
    // otherwise fall back to the no-op simulated backend (keyboard still works
    // via winit events regardless of which input backend is chosen here).
    #[cfg(feature = "gilrs")]
    let input_backend = input::GilrsBackend::new();
    #[cfg(not(feature = "gilrs"))]
    let input_backend = input::SimulatedBackend::new();

    App::run(
        GameState::new(),
        input_backend,
        |window| WgpuDriver::new(window),
        |state, events| { state.tick(events); },
        |state, driver| {
            // App has already called begin_frame() for us.
            game::sandbox::draw_scene(driver, state.fps(), state.player_pos());
        },
    );
}
