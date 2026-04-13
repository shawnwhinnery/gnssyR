# gnssyR

A local couch co-op 2D game (up to 4 players) with a web IO-game aesthetic. Written in Rust.

## Crates

| Crate | Role |
|-------|------|
| `gfx` | Backend-agnostic vector graphics layer — paths, shapes, styles, tessellation |
| `gfx-wgpu` | GPU driver (production) via `wgpu` |
| `gfx-software` | CPU/headless driver for tests — exposes a pixel buffer |
| `input` | Unified input for up to 4 players; `SimulatedBackend` for tests |
| `window` | `App::run` entry point, winit event loop |
| `game` | Game loop and mechanics (placeholder) |

## Running

```sh
cargo run
```

## Testing

All tests are headless — no GPU or display required.

```sh
cargo test
```

### Scene snapshot tests

The `game` crate renders the GFX showcase scene via `SoftwareDriver` and compares the output pixel-for-pixel against a golden file at `crates/game/tests/snapshots/gfx_scene.bin`.

**First run / after an intentional visual change** — regenerate the golden file:

```sh
UPDATE_SNAPSHOTS=1 cargo test -p game
```

Commit the updated `gfx_scene.bin` alongside your code changes to lock in the new baseline.

**Investigating a failure** — when the test fails it writes the actual output to `gfx_scene.actual.bin` in the same directory so you can compare the two files side-by-side.
# gnssyR
