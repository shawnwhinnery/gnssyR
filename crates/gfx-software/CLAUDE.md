# CLAUDE.md — gfx-software

## Scope

`gfx-software` is the CPU/headless `GraphicsDriver` implementation used for deterministic tests and CI environments without GPU access.

## Source of Truth

- Spec: `crates/gfx-software/index.md`
- Tests plan: `crates/gfx-software/test.md`
- Key modules: `crates/gfx-software/src/driver.rs`, `crates/gfx-software/src/raster.rs`

## Non-Negotiable Invariants

- Must satisfy the full `gfx::GraphicsDriver` contract.
- `SoftwareDriver::headless(width, height)` allocates an in-memory framebuffer without requiring a window.
- `present()` is a no-op in headless mode.
- `pixels()` returns the current framebuffer in packed ARGB `u32` values for assertions.
- Raster output should remain deterministic enough for snapshot/pixel tests (allow only documented tolerances).

## Editing Guidance

- Keep this crate independent of GPU/window system assumptions.
- Preserve clear behavior (`clear` overwrites full target) and predictable mesh draw order.
- Prefer correctness and testability over optimization unless a benchmark/regression requires performance work.
- Any rasterization math changes should include tests for geometric coverage and transform behavior.

## Validation

- Run crate tests: `cargo test -p gfx-software`
- Verify integration usage where relevant: `cargo test -p game`
