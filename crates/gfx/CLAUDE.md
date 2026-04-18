# CLAUDE.md — gfx

## Scope

`gfx` is the backend-agnostic vector graphics contract layer. It defines the `GraphicsDriver` trait and shared rendering types used by both `gfx-wgpu` and `gfx-software`.

## Source of Truth

- Spec: `crates/gfx/index.md`
- Tests plan: `crates/gfx/test.md`
- Key modules: `crates/gfx/src/driver.rs`, `crates/gfx/src/scene.rs`, `crates/gfx/src/path/`, `crates/gfx/src/shape.rs`, `crates/gfx/src/style.rs`

## Non-Negotiable Invariants

- Follow the frame lifecycle contract exactly: `begin_frame` -> draw/upload/clear work -> `end_frame` -> `present`.
- Mesh handles are frame-scoped and must not be used across `begin_frame` boundaries.
- Scene rendering must bracket all draw work with one `begin_frame` and one `end_frame`.
- Transform composition order must remain `a.then(b).apply(p) == b.apply(a.apply(p))`.
- Parameterized path APIs (`point_at`, `tangent_at`, `trim`, `split_at`) are arc-length based over `[0, 1]`.

## Editing Guidance

- Preserve backend neutrality; never import concrete backend crates here.
- When adjusting shape/path behavior, update both implementation and tests/spec expectations.
- Keep undefined behavior explicitly undefined where the spec says so; do not silently add guarantees.
- Prefer small, composable APIs used by both drivers and the scene graph.

## Validation

- Run crate tests after changes: `cargo test -p gfx`
- If shared contracts changed, run dependent crates too: `cargo test -p gfx-software -p gfx-wgpu -p game`
