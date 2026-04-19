/// Scenes used exclusively for automated bitmap-comparison snapshot tests.
///
/// These are not production game scenes — they exist to exercise specific
/// subsystems (gfx primitives, physics rendering, etc.) in a controlled,
/// deterministic way so that pixel-level regressions are caught early.
///
/// Each scene in this module implements [`game::scenes::Scene`] so the test
/// harness can call `draw` through the same interface used by the runtime.
pub mod gfx_showcase;
