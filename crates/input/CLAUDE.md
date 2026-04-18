# CLAUDE.md — input

## Scope

`input` defines the unified local multiplayer input abstraction and backends. It normalizes hardware events into `InputEvent` values for up to 4 local players.

## Source of Truth

- Spec: `crates/input/index.md`
- Key modules: `crates/input/src/event.rs`, `crates/input/src/backend.rs`, `crates/input/src/gilrs_backend.rs`, `crates/input/src/simulated.rs`, `crates/input/src/player.rs`

## Non-Negotiable Invariants

- `InputBackend::poll()` drains pending events in occurrence order.
- Player slots are `P1..P4`; gamepads fill slots by connection order and reuse the lowest free slot on reconnect.
- Keyboard/mouse fallback maps to `P1`.
- Axis deadzone in `GilrsBackend` is `0.1`, clamped to `0.0` below threshold.
- Event model includes both relative and absolute cursor data (`MouseMove` and `CursorMoved` with NDC coordinates).

## Editing Guidance

- Keep event semantics explicit and backend-independent at the API boundary.
- Preserve deterministic queue behavior in `SimulatedBackend`; this underpins automated tests.
- Avoid backend-specific leakage into shared enums/traits unless reflected in the spec.
- If changing `InputEvent` variants or mapping rules, update docs/tests in the same change.

## Validation

- Run crate tests: `cargo test -p input`
- Run integration consumers when mappings change: `cargo test -p window -p game`
