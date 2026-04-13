# Spec: Window / App Loop

## Purpose
Define the behaviour of the `App::run` entry point — the winit event loop that owns
the window lifetime, drives the frame tick, and coordinates the driver and input backend.

Crate: `window`.

---

## App::run

Signature (conceptual):

```
App::run(state, input_backend, graphics_driver, tick_fn, render_fn)
```

- Blocks the calling thread until the window is closed or the OS requests exit
- Owns the winit `EventLoop` and `Window` for its entire duration

### Per-frame sequence (in order)
1. Poll OS events (resize, close request, keyboard/mouse raw events)
2. Forward raw keyboard/mouse events to the input backend
3. Call `input_backend.poll()` to collect all `InputEvent`s for this frame
4. Call `tick_fn(&mut state, events)` — update game state
5. Call `driver.begin_frame()`
6. Call `render_fn(&state, &mut driver)` — submit draw calls
7. Call `driver.end_frame()`
8. Call `driver.present()`

### Close / exit
- A close request (OS close button or `Escape` key by convention) exits the loop cleanly
- `tick_fn` and `render_fn` are not called after a close request is received
- The driver and input backend are dropped in order after the loop exits

### Resize
- On window resize, the driver is notified before the next frame begins
- `driver.surface_size()` reflects the new dimensions from that frame onward
- The tick and render functions are not called during the resize event itself

---

## Frame Timing

- The loop runs as fast as the OS allows (no artificial frame cap at this layer)
- Frame pacing (vsync, fixed timestep) is the responsibility of the `tick_fn`, not `App`
- `App` does not inject delta-time; callers manage their own timers if needed

---

## Headless / Test Mode

- `App::run` requires a real OS window and is not suitable for headless tests
- Headless tests bypass `App` entirely: they call `tick_fn` and `render_fn` directly
  with a `SimulatedBackend` and `SoftwareDriver`
- The `tick_fn` / `render_fn` signatures are designed to be callable outside of `App`

---

## Constraints

- `App` must not hold any game state itself — it is a pure loop driver
- `App` must not know about specific driver or input backend types (generic over traits)
- The window handle must be accessible to construct the `WgpuDriver` before `run` is called
