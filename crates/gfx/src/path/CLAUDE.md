# CLAUDE.md — gfx/src/path/

## Purpose

The path subsystem: constructing, evaluating, and tessellating 2D vector paths. Consumed by both `gfx-software` (CPU raster) and `gfx-wgpu` (GPU buffer upload).

## Files

| File | Responsibility |
|------|---------------|
| `mod.rs` | `Path` type; `PathSegment` enum; module re-exports |
| `builder.rs` | `PathBuilder` — fluent segment-by-segment construction: `move_to`, `line_to`, `cubic_bezier_to`, `arc_to`, `close` |
| `parametric.rs` | Arc-length parametric API: `point_at(t)`, `tangent_at(t)`, `trim(t0, t1)`, `split_at(t)` over `t ∈ [0, 1]` |
| `tessellate.rs` | `tessellate(path, style) -> (Vec<Vertex>, Vec<u32>)` — converts the path to a triangle mesh |

## Parametric API Invariants

- `t` is **arc-length normalized**: `t = 0.5` is the geometric midpoint of the path, not the parameter midpoint.
- `point_at(0.0)` returns the path start; `point_at(1.0)` returns the path end.
- Bezier curves are subdivided to a target chord-length tolerance before arc-length tables are built.

## Tessellation

`tessellate` produces vertex/index arrays consumed directly by both drivers:
- `gfx-software`: fed into the CPU scan-converter in `raster.rs`
- `gfx-wgpu`: uploaded to a GPU vertex/index buffer in `buffer.rs`

The tessellator must remain deterministic for the same path input (snapshot tests depend on it).

## Closed vs Open Paths

- Calling `close()` on `PathBuilder` adds an implicit closing segment back to the move-to point.
- Open paths have no closing segment — the stroke ends are open caps.
- `tessellate` handles both cases via `PathSegment::Close` presence check.
