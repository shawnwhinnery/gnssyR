// CPU triangle rasteriser — barycentric with colour interpolation.
//
// Called by SoftwareDriver::end_frame for each recorded draw call.
// The pixel buffer stores ARGB packed u32: `(a<<24)|(r<<16)|(g<<8)|b`.
//
// Coordinate mapping (clip → pixel):
//   pixel_x = (clip_x + 1.0) / 2.0 * width
//   pixel_y = (1.0 - clip_y) / 2.0 * height   (y-axis is flipped)

use gfx::driver::Vertex;
use glam::{Mat3, Vec2};

/// Rasterise a triangle mesh into `pixels`.
///
/// `transform` is the 2D affine transform applied to each vertex position
/// before projection to pixel space. `tint` is multiplied with per-vertex
/// colour before writing.
pub(crate) fn rasterize(
    pixels:    &mut [u32],
    width:     u32,
    height:    u32,
    vertices:  &[Vertex],
    indices:   &[u32],
    transform: Mat3,
    tint:      [f32; 4],
) {
    let tri_count = indices.len() / 3;
    for t in 0..tri_count {
        let i0 = indices[t * 3]     as usize;
        let i1 = indices[t * 3 + 1] as usize;
        let i2 = indices[t * 3 + 2] as usize;

        if i0 >= vertices.len() || i1 >= vertices.len() || i2 >= vertices.len() {
            continue;
        }

        let v0 = &vertices[i0];
        let v1 = &vertices[i1];
        let v2 = &vertices[i2];

        // Transform clip-space positions.
        let p0 = clip_to_screen(transform_pos(transform, v0.position), width, height);
        let p1 = clip_to_screen(transform_pos(transform, v1.position), width, height);
        let p2 = clip_to_screen(transform_pos(transform, v2.position), width, height);

        // Tinted colours.
        let c0 = tint_color(v0.color, tint);
        let c1 = tint_color(v1.color, tint);
        let c2 = tint_color(v2.color, tint);

        rasterize_triangle(pixels, width, height, p0, p1, p2, c0, c1, c2);
    }
}

// ---------------------------------------------------------------------------
// Internals
// ---------------------------------------------------------------------------

fn transform_pos(t: Mat3, pos: [f32; 2]) -> Vec2 {
    t.transform_point2(Vec2::new(pos[0], pos[1]))
}

fn clip_to_screen(clip: Vec2, width: u32, height: u32) -> Vec2 {
    Vec2::new(
        (clip.x + 1.0) / 2.0 * width  as f32,
        (1.0 - clip.y) / 2.0 * height as f32,
    )
}

fn tint_color(color: [f32; 4], tint: [f32; 4]) -> [f32; 4] {
    [
        color[0] * tint[0],
        color[1] * tint[1],
        color[2] * tint[2],
        color[3] * tint[3],
    ]
}

fn pack_argb(r: f32, g: f32, b: f32, a: f32) -> u32 {
    let ri = (r.clamp(0.0, 1.0) * 255.0) as u32;
    let gi = (g.clamp(0.0, 1.0) * 255.0) as u32;
    let bi = (b.clamp(0.0, 1.0) * 255.0) as u32;
    let ai = (a.clamp(0.0, 1.0) * 255.0) as u32;
    (ai << 24) | (ri << 16) | (gi << 8) | bi
}

/// Rasterise a single triangle using a scanline / bounding-box approach
/// with barycentric interpolation of vertex colours.
fn rasterize_triangle(
    pixels: &mut [u32],
    width:  u32,
    height: u32,
    p0: Vec2, p1: Vec2, p2: Vec2,
    c0: [f32; 4], c1: [f32; 4], c2: [f32; 4],
) {
    // Bounding box clamped to the framebuffer.
    let min_x = p0.x.min(p1.x).min(p2.x).floor().max(0.0) as i32;
    let min_y = p0.y.min(p1.y).min(p2.y).floor().max(0.0) as i32;
    let max_x = (p0.x.max(p1.x).max(p2.x).ceil() as i32).min(width  as i32 - 1);
    let max_y = (p0.y.max(p1.y).max(p2.y).ceil() as i32).min(height as i32 - 1);

    let denom = edge(p0, p1, p2);
    if denom.abs() < 1e-6 { return; } // degenerate triangle

    for py in min_y..=max_y {
        for px in min_x..=max_x {
            let p = Vec2::new(px as f32 + 0.5, py as f32 + 0.5);

            let w0 = edge(p1, p2, p) / denom;
            let w1 = edge(p2, p0, p) / denom;
            let w2 = edge(p0, p1, p) / denom;

            if w0 < 0.0 || w1 < 0.0 || w2 < 0.0 { continue; }

            let r = w0 * c0[0] + w1 * c1[0] + w2 * c2[0];
            let g = w0 * c0[1] + w1 * c1[1] + w2 * c2[1];
            let b = w0 * c0[2] + w1 * c1[2] + w2 * c2[2];
            let a = w0 * c0[3] + w1 * c1[3] + w2 * c2[3];

            pixels[py as usize * width as usize + px as usize] = pack_argb(r, g, b, a);
        }
    }
}

/// Signed area of the parallelogram formed by (a→b, a→c) — also the edge
/// function used for barycentric weight computation.
#[inline]
fn edge(a: Vec2, b: Vec2, c: Vec2) -> f32 {
    (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)
}
