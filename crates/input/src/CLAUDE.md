# CLAUDE.md — input/src/

## Purpose

4-player input normalization. Abstracts over keyboard/mouse and gamepads into a unified `InputEvent` stream consumed by the game tick loop.

## Files

| File | Responsibility |
|------|---------------|
| `lib.rs` | Crate root — re-exports public types |
| `event.rs` | `InputEvent` enum — all normalized events; `PlayerSlot` (`P1`–`P4`); axis and button variants; `MouseMove` (relative) and `CursorMoved` (NDC absolute) |
| `backend.rs` | `InputBackend` trait — `poll() -> Vec<InputEvent>`; `connected_gamepads() -> usize` |
| `player.rs` | Player slot assignment: gamepads fill P1–P4 by connection order; lowest free slot reused on reconnect |
| `gilrs_backend.rs` | `GilrsBackend` — real gamepad input via gilrs; axis deadzone 0.1 clamped to 0.0 below threshold |
| `simulated.rs` | `SimulatedBackend` — deterministic FIFO event queue for tests; `push(event)` / `poll()` drains in order |

## Invariants

- `poll()` drains events in occurrence order — no reordering or deduplication.
- Keyboard/mouse always maps to `P1` regardless of connected gamepads.
- Gamepad slots reuse the lowest free slot on reconnect.
- `GilrsBackend`: axis values below 0.1 are clamped to 0.0 (deadzone).
- `SimulatedBackend` queue is FIFO and fully deterministic — this underpins all automated tests.
- `InputEvent` variants must remain backend-agnostic — no gilrs types appear in the public enum.
- If `InputEvent` variants or mapping rules change, update spec/tests in the same change.
