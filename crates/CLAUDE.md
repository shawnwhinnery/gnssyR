# CLAUDE.md — crates/

## Crate Landscape

Seven crates compose the full stack, ordered by dependency depth (fewest deps first):

```
gfx                ← pure trait + shared types; no GPU, no OS
  ├── gfx-wgpu     ← GraphicsDriver on wgpu (production GPU path)
  └── gfx-software ← GraphicsDriver in CPU memory (headless / tests)
input              ← 4-player event normalization; SimulatedBackend for tests
physics            ← 2D rigid-body simulation; glam only, zero gfx dep
window             ← App loop (winit); generic over gfx + input traits
game               ← all gameplay: World, scenes, weapons, enemies, NPCs
```

## Responsibility Boundaries

| Crate | Owns | Must NOT |
|-------|------|----------|
| `gfx` | `GraphicsDriver` trait, `Color`, `Transform`, `Style`, `Path`, `Shape`, `Scene` | Import any backend crate |
| `gfx-wgpu` | GPU rendering, wgpu surface, egui integration | Game logic, headless alloc |
| `gfx-software` | CPU framebuffer, pixel-exact rasterization | Window / OS dependencies |
| `input` | `InputEvent` enum, `InputBackend` trait, player slot assignment | Game state or rendering |
| `physics` | `PhysicsWorld`, rigid bodies, SAT narrowphase, collision layers | Any gfx dependency |
| `window` | `App::run` / `App::run_with_ui`, frame lifecycle | Store game state |
| `game` | Scenes, world simulation, weapons, enemies, NPCs, UI | Import `gfx-wgpu` directly (uses trait objects) |

## Dependency Graph (direct)

```
game        → gfx, input, physics, window, gfx-software (dev), egui
window      → gfx, input, winit, gfx-wgpu (feature-gated)
gfx-wgpu    → gfx, wgpu, egui-wgpu
gfx-software → gfx
input       → gilrs (optional feature)
physics     → glam
gfx         → glam
```

## Testing Strategy

All crates are exercisable headlessly — no GPU, no real window required:

- `gfx-software` provides `SoftwareDriver::headless(w, h)` → in-memory framebuffer
- `input` provides `SimulatedBackend` → deterministic FIFO event injection
- `App::run_frames(n)` drives N tick+render cycles without winit
- `physics` tests run standalone with zero graphics
- `game` integration tests compose `SoftwareDriver` + `SimulatedBackend`

## Spec-First Workflow

Each crate follows: **`index.md`** (spec) → **`test.md`** (test checklist) → **`src/`** (implementation).
See each crate's own `CLAUDE.md` for crate-level invariants, and `src/CLAUDE.md` for file-level detail.

## Quick Commands

```bash
cargo test                             # all crates
cargo test -p <crate>                  # single crate
UPDATE_SNAPSHOTS=1 cargo test -p game  # regenerate visual golden snapshots
```
