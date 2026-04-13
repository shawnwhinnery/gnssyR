pub mod backend;
pub mod event;
pub mod player;
pub mod simulated;

#[cfg(feature = "gilrs")]
pub mod gilrs_backend;

pub use backend::InputBackend;
pub use event::InputEvent;
pub use player::PlayerId;
pub use simulated::SimulatedBackend;

#[cfg(feature = "gilrs")]
pub use gilrs_backend::GilrsBackend;
