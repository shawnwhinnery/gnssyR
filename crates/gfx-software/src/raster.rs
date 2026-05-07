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
    pixels: &mut [u32],
    width: u32,
    height: u32,
    vertices: &[Vertex],
    indices: &[u32],
    transform: Mat3,
    tint: [f32; 4],
) {
    let tri_count = indices.len() / 3;
    for t in 0..tri_count {
        let i0 = indices[t * 3] as usize;
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
        (clip.x + 1.0) / 2.0 * width as f32,
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
    width: u32,
    height: u32,
    p0: Vec2,
    p1: Vec2,
    p2: Vec2,
    c0: [f32; 4],
    c1: [f32; 4],
    c2: [f32; 4],
) {
    // Bounding box clamped to the framebuffer.
    let min_x = p0.x.min(p1.x).min(p2.x).floor().max(0.0) as i32;
    let min_y = p0.y.min(p1.y).min(p2.y).floor().max(0.0) as i32;
    let max_x = (p0.x.max(p1.x).max(p2.x).ceil() as i32).min(width as i32 - 1);
    let max_y = (p0.y.max(p1.y).max(p2.y).ceil() as i32).min(height as i32 - 1);

    let denom = edge(p0, p1, p2);
    if denom.abs() < 1e-6 {
        return;
    } // degenerate triangle

    for py in min_y..=max_y {
        for px in min_x..=max_x {
            let p = Vec2::new(px as f32 + 0.5, py as f32 + 0.5);

            let w0 = edge(p1, p2, p) / denom;
            let w1 = edge(p2, p0, p) / denom;
            let w2 = edge(p0, p1, p) / denom;

            if w0 < 0.0 || w1 < 0.0 || w2 < 0.0 {
                continue;
            }

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

/// Rasterise a textured quad (full bitmap) with nearest-neighbour sampling and
/// Porter-Duff "over" blending into `pixels`.
pub(crate) fn raster_bitmap(
    pixels: &mut [u32],
    fb_w: u32,
    fb_h: u32,
    tex_pixels: &[u32],
    tex_w: u32,
    tex_h: u32,
    transform: Mat3,
    tint: [f32; 4],
) {
    if tex_w == 0 || tex_h == 0 {
        return;
    }

    // Clip-space corners + UV (row 0 = top of texture), matching `WgpuDriver`.
    let corners = [
        ([-1.0f32, -1.0f32], [0.0f32, 1.0f32]),
        ([1.0, -1.0], [1.0, 1.0]),
        ([1.0, 1.0], [1.0, 0.0]),
        ([-1.0, 1.0], [0.0, 0.0]),
    ];

    let mut pscr = [Vec2::ZERO; 4];
    let mut uvs = [[0f32; 2]; 4];
    for i in 0..4 {
        let clip = transform_pos(transform, corners[i].0);
        pscr[i] = clip_to_screen(clip, fb_w, fb_h);
        uvs[i] = corners[i].1;
    }

    rasterize_textured_triangle(
        pixels, fb_w, fb_h, tex_pixels, tex_w, tex_h, pscr[0], pscr[1], pscr[2], uvs[0], uvs[1],
        uvs[2], tint,
    );
    rasterize_textured_triangle(
        pixels, fb_w, fb_h, tex_pixels, tex_w, tex_h, pscr[0], pscr[2], pscr[3], uvs[0], uvs[2],
        uvs[3], tint,
    );
}

fn unpack_argb(p: u32) -> [f32; 4] {
    [
        ((p >> 16) & 0xFF) as f32 / 255.0,
        ((p >> 8) & 0xFF) as f32 / 255.0,
        (p & 0xFF) as f32 / 255.0,
        ((p >> 24) & 0xFF) as f32 / 255.0,
    ]
}

fn sample_tex(tex: &[u32], w: u32, h: u32, u: f32, v: f32) -> [f32; 4] {
    let wf = w as f32;
    let hf = h as f32;
    let tx = (u * (wf - 1.0).max(0.0)).round().clamp(0.0, wf - 1.0) as u32;
    let ty = (v * (hf - 1.0).max(0.0)).round().clamp(0.0, hf - 1.0) as u32;
    let idx = (ty * w + tx) as usize;
    if idx >= tex.len() {
        return [0.0, 0.0, 0.0, 0.0];
    }
    unpack_argb(tex[idx])
}

fn blend_over(dst: u32, src: [f32; 4]) -> u32 {
    let d = unpack_argb(dst);
    let sa = src[3].clamp(0.0, 1.0);
    let da = d[3].clamp(0.0, 1.0);
    let out_a = sa + da * (1.0 - sa);
    if out_a < 1e-6 {
        return 0;
    }
    let sp = [src[0] * sa, src[1] * sa, src[2] * sa];
    let dp = [d[0] * da, d[1] * da, d[2] * da];
    let op = [
        sp[0] + dp[0] * (1.0 - sa),
        sp[1] + dp[1] * (1.0 - sa),
        sp[2] + dp[2] * (1.0 - sa),
    ];
    pack_argb(op[0] / out_a, op[1] / out_a, op[2] / out_a, out_a)
}

fn rasterize_textured_triangle(
    pixels: &mut [u32],
    width: u32,
    height: u32,
    tex_pixels: &[u32],
    tex_w: u32,
    tex_h: u32,
    p0: Vec2,
    p1: Vec2,
    p2: Vec2,
    uv0: [f32; 2],
    uv1: [f32; 2],
    uv2: [f32; 2],
    tint: [f32; 4],
) {
    let min_x = p0.x.min(p1.x).min(p2.x).floor().max(0.0) as i32;
    let min_y = p0.y.min(p1.y).min(p2.y).floor().max(0.0) as i32;
    let max_x = (p0.x.max(p1.x).max(p2.x).ceil() as i32).min(width as i32 - 1);
    let max_y = (p0.y.max(p1.y).max(p2.y).ceil() as i32).min(height as i32 - 1);

    let denom = edge(p0, p1, p2);
    if denom.abs() < 1e-6 {
        return;
    }

    for py in min_y..=max_y {
        for px in min_x..=max_x {
            let p = Vec2::new(px as f32 + 0.5, py as f32 + 0.5);

            let w0 = edge(p1, p2, p) / denom;
            let w1 = edge(p2, p0, p) / denom;
            let w2 = edge(p0, p1, p) / denom;

            if w0 < 0.0 || w1 < 0.0 || w2 < 0.0 {
                continue;
            }

            let u = w0 * uv0[0] + w1 * uv1[0] + w2 * uv2[0];
            let v = w0 * uv0[1] + w1 * uv1[1] + w2 * uv2[1];

            let mut c = sample_tex(tex_pixels, tex_w, tex_h, u, v);
            c[0] *= tint[0];
            c[1] *= tint[1];
            c[2] *= tint[2];
            c[3] *= tint[3];

            let idx = py as usize * width as usize + px as usize;
            pixels[idx] = blend_over(pixels[idx], c);
        }
    }
}
