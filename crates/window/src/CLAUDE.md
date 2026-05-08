# CLAUDE.md — window/src/

## Purpose

App loop orchestration. Coordinates OS events, input polling, scene ticking, and rendering. Fully generic over the graphics driver and input backend — no concrete backend coupling here.

## Files

| File | Responsibility |
|------|---------------|
| `lib.rs` | Crate root — exports `App`, `EguiRenderer` |
| `app.rs` | `App<D, I>` generic struct; `App::run` (no egui); `App::run_with_ui` (egui path); `App::run_frames(n)` (headless test driver) |
| `egui_renderer.rs` | `EguiRenderer` trait — `begin_ui_frame`, `end_ui_frame`, `render_ui`; implemented by `WgpuDriver` in `gfx-wgpu` |

## Frame Order (`App::run_with_ui`)

```
OS events
  → egui input consumed first (events marked consumed are NOT forwarded to game)
  → input.poll() → [InputEvent]
  → tick_fn(events)
  → driver.begin_frame()
  → render_fn(driver)
  → egui: begin_ui_frame → draw_ui(ctx) → end_ui_frame → render_ui(driver)
  → driver.end_frame()
  → driver.present()
```

## Headless Test Path (`App::run_frames`)

`App::run_frames(n, tick_fn, render_fn)` drives exactly `n` tick+render cycles without winit, egui, or a real window. Used in `game` integration tests with `SoftwareDriver` + `SimulatedBackend`.

## Invariants

- `App` stores no game state — only loop machinery and driver/input handles.
- `App` must not import concrete backend crates; it is generic over `D: GraphicsDriver` and `I: InputBackend`.
- Close requests halt tick/render and exit cleanly — no further calls after a close event.
- Resize events update driver state (`driver.resize(w, h)`) before the next `begin_frame`.
- egui input consumption happens before game input; do not reorder these.
- `tick_fn` and `render_fn` closures are the only coupling point to game code.
