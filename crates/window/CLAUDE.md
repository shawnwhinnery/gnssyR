# CLAUDE.md — window

## Scope

`window` owns the app loop entry point (`App::run`) and coordinates OS events, input polling, ticking, and rendering.

## Source of Truth

- Spec: `crates/window/index.md`
- Tests plan: `crates/window/test.md`
- Key modules: `crates/window/src/app.rs`, `crates/window/src/lib.rs`

## Non-Negotiable Invariants

- `App` remains generic over input and graphics traits; no concrete backend coupling.
- Per-frame order must remain: OS events -> input forwarding -> `poll()` -> `tick_fn` -> `begin_frame` -> `render_fn` -> `end_frame` -> `present`.
- Close requests stop future tick/render calls and exit cleanly.
- Resize notifications update driver state before the next frame.
- `App` should not store game state; it only drives loop orchestration.

## Editing Guidance

- Keep this crate focused on lifecycle orchestration, not gameplay logic.
- Preserve testability by maintaining separable `tick_fn` / `render_fn` signatures.
- Be careful with event ordering; subtle reorderings can break deterministic behavior.
- When changing loop behavior, sync spec and test docs in the same PR.

## Validation

- Run crate tests: `cargo test -p window`
- Run higher-level checks if loop contracts change: `cargo test -p game`
