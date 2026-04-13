# Window Crate Tests

## Design Note

`App::run` blocks on a real OS window and is not testable headlessly.
Two headless entry points cover the two construction paths:

- `App::run_frames(state, input, driver, tick, render, n)` — driver passed
  directly (no window needed). Tests the frame loop logic.
- `App::run_frames_with_factory(state, input, make_driver, tick, render, n)` —
  driver created by a factory closure, mirroring the deferred-construction path
  of `App::run` where the driver is built after the window exists. Tests that
  the factory is invoked and the resulting driver is used.

All tests use `SpyDriver` (defined in tests) and `SimulatedBackend` from the
`input` crate.

The per-frame sequence under test (matches spec):
1. `input_backend.poll()` → events
2. `tick_fn(&mut state, events)`
3. `driver.begin_frame()`
4. `render_fn(&state, &mut driver)`
5. `driver.end_frame()`
6. `driver.present()`

---

## Tests

### 1. `frame_sequence_begin_render_end_present`
**What**: A single frame calls the driver in the correct order.
**How**: SpyDriver appends a label on each method call. After `run_frames(n=1)`,
assert the call log is exactly `["begin_frame", "end_frame", "present"]` and
that render_fn was invoked between begin and end.

### 2. `input_events_delivered_to_tick`
**What**: Events pushed into `SimulatedBackend` arrive in `tick_fn`.
**How**: Push `Button { South, pressed: true }` and `MouseMove { dx: 1.0, dy: 2.0 }`
before calling `run_frames(n=1)`. Assert tick received exactly those two events.

### 3. `empty_poll_delivers_empty_events`
**What**: When the backend has no queued events, tick receives an empty Vec.
**How**: Push nothing. Call `run_frames(n=1)`. Assert tick receives `vec![]`.

### 4. `tick_state_mutation_visible_in_render`
**What**: State mutated by tick is visible to render in the same frame.
**How**: State is `{ counter: u32 }`. tick increments it. render records the
value it sees. After `run_frames(n=1)`, assert render saw `counter == 1`.

### 5. `multiple_frames_call_counts`
**What**: Over N frames, driver lifecycle methods are each called exactly N times.
**How**: Run 3 frames. Assert `begin_frame`, `end_frame`, and `present` each
appear exactly 3 times in the SpyDriver call log.

### 6. `events_cleared_between_frames`
**What**: Events are drained each frame; they don't bleed into the next frame.
**How**: Push one event before frame 1, push nothing before frame 2.
Use a tick that appends received events to an outer Vec per frame.
Assert frame 1 received 1 event and frame 2 received 0 events.

### 7. `render_sees_cumulative_state`
**What**: State accumulates correctly across frames.
**How**: tick increments counter each frame. render records the counter value.
After `run_frames(n=3)`, assert render saw values `[1, 2, 3]` in order.

### 8. `factory_is_called_to_create_driver`
**What**: `run_frames_with_factory` actually invokes the factory closure.
**Why this test exists**: The original `App::run` took a pre-built driver,
making it impossible for GPU drivers to receive the window handle at
construction time (the window didn't exist yet). This test encodes the
deferred-construction contract so that regression is impossible.
**How**: Pass a factory that sets a flag, then assert the flag is true.

### 9. `factory_driver_is_used_for_frames`
**What**: The driver returned by the factory is the one that receives
`begin_frame` / `end_frame` / `present` — not some default or discarded instance.
**How**: Factory returns a `LoggingDriver`. After `run_frames_with_factory(n=2)`,
assert the log contains exactly two full begin/end/present sequences.
