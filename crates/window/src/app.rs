use crate::egui_renderer::EguiRenderer;
use gfx::driver::GraphicsDriver;
use input::backend::InputBackend;
use input::event::{Button, InputEvent};
use input::player::PlayerId;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Owns the winit event loop and drives the main game loop.
///
/// `App::run` blocks until the window is closed. Each frame:
///   1. Poll input backend for [`InputEvent`]s
///   2. Call `tick` with the collected events
///   3. Call `driver.begin_frame()`
///   4. Call `render` with the driver
///   5. Call `driver.end_frame()`
///   6. Call `driver.present()`
pub struct App;

impl App {
    /// Run the application against a real OS window. Blocks until closed.
    ///
    /// `make_driver` is called once after the window is created so that
    /// GPU drivers (e.g. `WgpuDriver`) can obtain the window handle.
    pub fn run<S, I, D, F, T, R>(state: S, input: I, make_driver: F, tick: T, render: R)
    where
        F: FnOnce(&Window) -> D + 'static,
        I: InputBackend + 'static,
        D: GraphicsDriver + 'static,
        T: FnMut(&mut S, Vec<InputEvent>) + 'static,
        R: FnMut(&S, &mut dyn GraphicsDriver) + 'static,
        S: 'static,
    {
        let event_loop = EventLoop::new().expect("failed to create event loop");
        let mut app = WinitApp {
            state,
            input,
            make_driver: Some(make_driver),
            driver: None,
            tick,
            render,
            window: None,
            pending_keys: Vec::new(),
            win_size: (1, 1),
        };
        event_loop.run_app(&mut app).expect("event loop error");
    }

    /// Run the application with an egui overlay. Blocks until closed.
    ///
    /// This is an opt-in variant of [`App::run`] for drivers that implement
    /// [`EguiRenderer`]. The egui context is provided by the caller so they can
    /// hold a clone for use inside `render` (to call `scene.draw_ui(&ctx)`).
    ///
    /// Frame order (per CLAUDE invariant, with egui added):
    ///   1. OS events → egui_winit + game input
    ///   2. `ctx.begin_pass(raw_input)`
    ///   3. `tick(state, events)`
    ///   4. `driver.begin_frame()`
    ///   5. `render(state, driver)` — callers call `scene.draw_ui(&ctx_clone)` here
    ///   6. `ctx.end_pass()` → tessellate
    ///   7. `driver.prepare_egui(primitives, textures, screen)`
    ///   8. `driver.end_frame()` — renders game + egui together
    ///   9. `driver.present()`
    pub fn run_with_ui<S, I, D, F, T, R>(
        state: S,
        input: I,
        make_driver: F,
        tick: T,
        render: R,
        egui_ctx: egui::Context,
    ) where
        F: FnOnce(&Window) -> D + 'static,
        I: InputBackend + 'static,
        D: GraphicsDriver + EguiRenderer + 'static,
        T: FnMut(&mut S, Vec<InputEvent>) + 'static,
        R: FnMut(&S, &mut dyn GraphicsDriver) + 'static,
        S: 'static,
    {
        let event_loop = EventLoop::new().expect("failed to create event loop");
        let mut app = WinitAppEgui {
            state,
            input,
            make_driver: Some(make_driver),
            driver: None,
            tick,
            render,
            window: None,
            pending_keys: Vec::new(),
            win_size: (1, 1),
            egui_ctx,
            egui_winit: None,
        };
        event_loop.run_app(&mut app).expect("event loop error");
    }

    /// Drive exactly `n` frames without opening a window, constructing the
    /// driver via a factory closure. For testing only.
    ///
    /// Mirrors the deferred-construction path of `App::run` (where the driver
    /// is created after the window exists) without requiring a real window.
    /// Use this when testing code that depends on the driver being created
    /// lazily — e.g. to verify the factory is actually invoked.
    pub fn run_frames_with_factory<S, I, D, F, T, R>(
        state: S,
        input: I,
        make_driver: F,
        tick: T,
        render: R,
        n: usize,
    ) where
        F: FnOnce() -> D,
        I: InputBackend,
        D: GraphicsDriver,
        T: FnMut(&mut S, Vec<InputEvent>),
        R: FnMut(&S, &mut dyn GraphicsDriver),
    {
        let driver = make_driver();
        Self::run_frames(state, input, driver, tick, render, n);
    }

    /// Drive exactly `n` frames without opening a window. For testing only.
    ///
    /// The driver is passed directly (no window handle needed).
    pub fn run_frames<S, I, D, T, R>(
        mut state: S,
        mut input: I,
        mut driver: D,
        mut tick: T,
        mut render: R,
        n: usize,
    ) where
        I: InputBackend,
        D: GraphicsDriver,
        T: FnMut(&mut S, Vec<InputEvent>),
        R: FnMut(&S, &mut dyn GraphicsDriver),
    {
        for _ in 0..n {
            execute_frame(&mut state, &mut input, &mut driver, &mut tick, &mut render);
        }
    }
}

// ---------------------------------------------------------------------------
// Shared per-frame logic
// ---------------------------------------------------------------------------

fn execute_frame<S, I, D, T, R>(
    state: &mut S,
    input: &mut I,
    driver: &mut D,
    tick: &mut T,
    render: &mut R,
) where
    I: InputBackend,
    D: GraphicsDriver,
    T: FnMut(&mut S, Vec<InputEvent>),
    R: FnMut(&S, &mut dyn GraphicsDriver),
{
    let events = input.poll();
    tick(state, events);
    driver.begin_frame();
    render(state, driver);
    driver.end_frame();
    driver.present();
}

// ---------------------------------------------------------------------------
// Keyboard → abstract Button translation
// ---------------------------------------------------------------------------

fn translate_key(key: PhysicalKey) -> Option<Button> {
    let PhysicalKey::Code(code) = key else {
        return None;
    };
    Some(match code {
        KeyCode::KeyW | KeyCode::ArrowUp => Button::DPadUp,
        KeyCode::KeyS | KeyCode::ArrowDown => Button::DPadDown,
        KeyCode::KeyA | KeyCode::ArrowLeft => Button::DPadLeft,
        KeyCode::KeyD | KeyCode::ArrowRight => Button::DPadRight,
        KeyCode::Escape => Button::Key(input::event::KeyCode::Escape),
        _ => return None,
    })
}

// ---------------------------------------------------------------------------
// winit ApplicationHandler impl
// ---------------------------------------------------------------------------

struct WinitApp<S, I, D, F, T, R> {
    state: S,
    input: I,
    make_driver: Option<F>,
    // driver is declared before window so it is dropped first,
    // which is required when the driver holds a raw surface handle.
    driver: Option<D>,
    tick: T,
    render: R,
    window: Option<Window>,
    pending_keys: Vec<InputEvent>,
    // Physical pixel dimensions used to convert CursorMoved positions to NDC.
    win_size: (u32, u32),
}

impl<S, I, D, F, T, R> ApplicationHandler for WinitApp<S, I, D, F, T, R>
where
    F: FnOnce(&Window) -> D,
    I: InputBackend,
    D: GraphicsDriver,
    T: FnMut(&mut S, Vec<InputEvent>),
    R: FnMut(&S, &mut dyn GraphicsDriver),
{
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attrs = Window::default_attributes().with_title("gnssyR");
        let window = event_loop
            .create_window(attrs)
            .expect("failed to create window");

        let size = window.inner_size();
        self.win_size = (size.width.max(1), size.height.max(1));

        let driver = self.make_driver.take().unwrap()(&window);
        self.driver = Some(driver);
        self.window = Some(window);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                self.win_size = (new_size.width.max(1), new_size.height.max(1));
                if let Some(driver) = self.driver.as_mut() {
                    driver.resize(new_size.width, new_size.height);
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let (w, h) = self.win_size;
                let x = ((position.x as f32 / (w as f32 * 0.5)) - 1.0).clamp(-1.0, 1.0);
                let y = (1.0 - position.y as f32 / (h as f32 * 0.5)).clamp(-1.0, 1.0);
                self.pending_keys.push(InputEvent::CursorMoved { x, y });
            }
            WindowEvent::KeyboardInput { event, .. } => {
                // Don't re-emit OS key-repeat events — we track held state ourselves.
                if event.repeat {
                    return;
                }
                if let Some(button) = translate_key(event.physical_key) {
                    let pressed = event.state == ElementState::Pressed;
                    self.pending_keys.push(InputEvent::Button {
                        player: PlayerId::P1,
                        button,
                        pressed,
                    });
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(driver) = self.driver.as_mut() {
                    // Merge keyboard events with the input backend's events.
                    let mut events = self.input.poll();
                    events.extend(self.pending_keys.drain(..));
                    (self.tick)(&mut self.state, events);
                    driver.begin_frame();
                    (self.render)(&self.state, driver);
                    driver.end_frame();
                    driver.present();
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // Run as fast as the OS allows — request a redraw every iteration.
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

// ---------------------------------------------------------------------------
// Egui-aware winit app
// ---------------------------------------------------------------------------

struct WinitAppEgui<S, I, D, F, T, R> {
    state: S,
    input: I,
    make_driver: Option<F>,
    // driver is declared before window so it is dropped first,
    // which is required when the driver holds a raw surface handle.
    driver: Option<D>,
    tick: T,
    render: R,
    window: Option<Window>,
    pending_keys: Vec<InputEvent>,
    win_size: (u32, u32),
    egui_ctx: egui::Context,
    egui_winit: Option<egui_winit::State>,
}

impl<S, I, D, F, T, R> ApplicationHandler for WinitAppEgui<S, I, D, F, T, R>
where
    F: FnOnce(&Window) -> D,
    I: InputBackend,
    D: GraphicsDriver + EguiRenderer,
    T: FnMut(&mut S, Vec<InputEvent>),
    R: FnMut(&S, &mut dyn GraphicsDriver),
{
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attrs = Window::default_attributes().with_title("gnssyR");
        let window = event_loop
            .create_window(attrs)
            .expect("failed to create window");

        let size = window.inner_size();
        self.win_size = (size.width.max(1), size.height.max(1));

        let driver = self.make_driver.take().unwrap()(&window);
        self.driver = Some(driver);

        let egui_winit = egui_winit::State::new(
            self.egui_ctx.clone(),
            egui::ViewportId::ROOT,
            &window,
            None,
            None,
            None,
        );
        self.egui_winit = Some(egui_winit);
        self.window = Some(window);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        // Feed the event to egui first; if it consumes it, skip game input.
        if let (Some(egui_winit), Some(window)) =
            (self.egui_winit.as_mut(), self.window.as_ref())
        {
            let response = egui_winit.on_window_event(window, &event);
            if response.consumed {
                // Still handle structural events even when egui consumed input.
                match &event {
                    WindowEvent::CloseRequested => event_loop.exit(),
                    WindowEvent::Resized(new_size) => {
                        self.win_size = (new_size.width.max(1), new_size.height.max(1));
                        if let Some(driver) = self.driver.as_mut() {
                            driver.resize(new_size.width, new_size.height);
                        }
                    }
                    WindowEvent::RedrawRequested => {
                        if let Some(window) = &self.window {
                            window.request_redraw();
                        }
                    }
                    _ => {}
                }
                return;
            }
        }

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                self.win_size = (new_size.width.max(1), new_size.height.max(1));
                if let Some(driver) = self.driver.as_mut() {
                    driver.resize(new_size.width, new_size.height);
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let (w, h) = self.win_size;
                let x = ((position.x as f32 / (w as f32 * 0.5)) - 1.0).clamp(-1.0, 1.0);
                let y = (1.0 - position.y as f32 / (h as f32 * 0.5)).clamp(-1.0, 1.0);
                self.pending_keys.push(InputEvent::CursorMoved { x, y });
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.repeat {
                    return;
                }
                if let Some(button) = translate_key(event.physical_key) {
                    let pressed = event.state == ElementState::Pressed;
                    self.pending_keys.push(InputEvent::Button {
                        player: PlayerId::P1,
                        button,
                        pressed,
                    });
                }
            }
            WindowEvent::RedrawRequested => {
                if let (Some(driver), Some(egui_winit), Some(window)) = (
                    self.driver.as_mut(),
                    self.egui_winit.as_mut(),
                    self.window.as_ref(),
                ) {
                    let mut events = self.input.poll();
                    events.extend(self.pending_keys.drain(..));

                    let raw_input = egui_winit.take_egui_input(window);
                    self.egui_ctx.begin_pass(raw_input);

                    (self.tick)(&mut self.state, events);

                    driver.begin_frame();
                    (self.render)(&self.state, driver);

                    let full_output = self.egui_ctx.end_pass();
                    egui_winit.handle_platform_output(window, full_output.platform_output);

                    let pixels_per_point = self.egui_ctx.pixels_per_point();
                    let primitives = self
                        .egui_ctx
                        .tessellate(full_output.shapes, pixels_per_point);
                    driver.prepare_egui(
                        primitives,
                        full_output.textures_delta,
                        [self.win_size.0, self.win_size.1],
                        pixels_per_point,
                    );

                    driver.end_frame();
                    driver.present();
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}
