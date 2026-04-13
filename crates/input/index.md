# Spec: Input

## Purpose
Define the unified input abstraction for a couch co-op game with up to 4 local players.
All hardware differences (gamepad models, keyboard/mouse) are normalised into a single
`InputEvent` stream. Tests use `SimulatedBackend` to inject events without real hardware.

Crate: `input`.

---

## PlayerId

- Identifies a local player slot: P1 (0), P2 (1), P3 (2), P4 (3)
- Players are assigned slots in gamepad connection order
- Keyboard/mouse always maps to P1 as a fallback

---

## InputEvent

All input is expressed as one of these variants:

| Variant | When emitted |
|---------|-------------|
| `Button { player, button, pressed }` | A digital button is pressed or released |
| `Axis { player, axis, value }` | An analogue axis changes; `value ∈ [-1.0, 1.0]` |
| `MouseMove { dx, dy }` | Mouse cursor moves; delta in screen pixels |
| `GamepadConnected(player)` | A gamepad is recognised and assigned a slot |
| `GamepadDisconnected(player)` | A gamepad is removed |

### Button enum
Abstract buttons shared across all input devices:

- Face: `South`, `East`, `West`, `North`
- Shoulders: `LeftBumper`, `RightBumper`
- Trigger threshold: `LeftTrigger`, `RightTrigger`
- Thumbstick click: `LeftStick`, `RightStick`
- D-pad: `DPadUp`, `DPadDown`, `DPadLeft`, `DPadRight`
- Menu: `Start`, `Select`
- Keyboard fallback: `Key(KeyCode)`

### Axis enum
- `LeftX`, `LeftY` — left thumbstick
- `RightX`, `RightY` — right thumbstick
- `LeftTrigger`, `RightTrigger` — analogue triggers (0.0 … 1.0)

### KeyCode enum (keyboard fallback)
- `W`, `A`, `S`, `D` — movement
- `Up`, `Down`, `Left`, `Right` — arrow keys
- `Space`, `Enter`, `Escape`

---

## InputBackend Trait

```
InputBackend::poll() -> Vec<InputEvent>
```

- Drains all pending events accumulated since the last call
- Returns an empty `Vec` if no events are pending
- Called once per frame by the game loop
- Order within the returned `Vec` reflects the order events occurred

---

## GilrsBackend

- Wraps `gilrs::Gilrs` to translate hardware gamepad events into `InputEvent`
- Gamepads are assigned `PlayerId` slots in the order they connect
- The first gamepad connection emits `GamepadConnected(P1)`, second `P2`, etc.
- Disconnection emits `GamepadDisconnected` with the slot that was freed
- Reconnecting after disconnection reuses the lowest available slot
- Axis values below a deadzone threshold (0.1) are clamped to 0.0

---

## SimulatedBackend

- `SimulatedBackend::new()` — empty queue
- `push(event)` — enqueues a single event
- `push_all(events)` — enqueues multiple events
- `poll()` — drains and returns the queue; subsequent call returns empty `Vec`
- No timing dependency: all pushed events are returned on the very next `poll`
- Used in all automated tests in place of `GilrsBackend`

### Test cases
- `push` then `poll` returns the event
- `poll` with no prior `push` returns empty `Vec`
- `push` twice then `poll` returns both events in insertion order
- Two consecutive `poll` calls: first returns all events, second returns empty
- `push_all([e1, e2, e3])` then `poll` returns `[e1, e2, e3]` in order
