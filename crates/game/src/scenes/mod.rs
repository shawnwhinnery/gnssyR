pub mod sandbox;

use input::InputEvent;

/// Returned by [`Scene::tick`] to request a scene change.
pub enum SceneTransition {
    /// Discard the current scene and replace it with `next`.
    ///
    /// The current scene is dropped (RAII cleanup) before `next` begins.
    Replace(Box<dyn Scene>),
    /// Shut down the application.
    Quit,
}

/// Lifecycle contract for all game scenes.
///
/// # Lifecycle
///
/// 1. **Construction** (`Scene::new` or equivalent) — allocate all scene-owned
///    state: physics world, bodies, initial entity set. This is the init step.
/// 2. **`tick`** — advance the simulation one logical step. Return
///    `Some(SceneTransition)` to hand control to a different scene.
/// 3. **`draw`** — render the current frame (including any overlays such as a
///    pause menu). Called after every tick.
/// 4. **Destruction** (`drop`) — Rust's RAII releases scene-owned resources
///    automatically. No explicit `deinit` method is needed today; an async
///    asset-unload hook can be added to this trait when required.
pub trait Scene {
    fn tick(&mut self, events: &[InputEvent]) -> Option<SceneTransition>;
    fn draw(&self, driver: &mut dyn gfx::GraphicsDriver);
}
