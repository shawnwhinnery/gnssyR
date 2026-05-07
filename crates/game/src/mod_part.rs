use glam::Vec2;
use rand::Rng;

use crate::scrap::{crescent_verts, ScrapColor, ScrapShape};

const VERTS: usize = 12;
const BASE_RADIUS: f32 = 0.5;
const NOISE_FRACTION: f32 = 0.10;

pub struct ModPart {
    pub avg_color: [f32; 3],
    pub shape: Vec<Vec2>,
}

/// Produce a `ModPart` from a slice of `(color, shape, count)` contributions.
/// Returns `None` if the total count is zero.
pub fn forge(contributions: &[(ScrapColor, ScrapShape, u16)]) -> Option<ModPart> {
    let total: u32 = contributions.iter().map(|(_, _, n)| *n as u32).sum();
    if total == 0 {
        return None;
    }

    let avg_color = weighted_color(contributions, total);
    let shape = blended_shape(contributions, total);

    Some(ModPart { avg_color, shape })
}

// ---------------------------------------------------------------------------
// Color averaging
// ---------------------------------------------------------------------------

fn weighted_color(contributions: &[(ScrapColor, ScrapShape, u16)], total: u32) -> [f32; 3] {
    let mut rgb = [0.0f32; 3];
    for (color, _, count) in contributions {
        let w = *count as f32 / total as f32;
        let c = color_rgb(*color);
        rgb[0] += c[0] * w;
        rgb[1] += c[1] * w;
        rgb[2] += c[2] * w;
    }
    rgb
}

fn color_rgb(color: ScrapColor) -> [f32; 3] {
    match color {
        ScrapColor::Red => [0.90, 0.15, 0.15],
        ScrapColor::Orange => [0.95, 0.50, 0.10],
        ScrapColor::Yellow => [0.95, 0.90, 0.10],
        ScrapColor::Green => [0.10, 0.85, 0.20],
        ScrapColor::Cyan => [0.10, 0.85, 0.90],
        ScrapColor::Blue => [0.15, 0.30, 0.95],
        ScrapColor::Purple => [0.65, 0.10, 0.90],
        ScrapColor::Pink => [0.95, 0.35, 0.75],
    }
}

// ---------------------------------------------------------------------------
// Shape blending
// ---------------------------------------------------------------------------

/// Resample a shape's perimeter to exactly VERTS evenly-spaced points.
fn shape_verts(shape: ScrapShape) -> [Vec2; VERTS] {
    // Canonical vertices at unit radius for each shape type.
    let raw: Vec<Vec2> = match shape {
        ScrapShape::Diamond => vec![
            Vec2::new(0.0, 1.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(0.0, -1.0),
            Vec2::new(-1.0, 0.0),
        ],
        ScrapShape::Circle => (0..VERTS)
            .map(|i| {
                let a = std::f32::consts::TAU * i as f32 / VERTS as f32;
                Vec2::new(a.cos(), a.sin())
            })
            .collect(),
        ScrapShape::Crescent => crescent_verts(1.0),
        ScrapShape::Triangle => vec![
            Vec2::new(0.0, 1.0),
            Vec2::new(-0.866, -0.5),
            Vec2::new(0.866, -0.5),
        ],
    };

    // If already VERTS points (circle), copy directly.
    if raw.len() == VERTS {
        let mut out = [Vec2::ZERO; VERTS];
        out.copy_from_slice(&raw);
        return out;
    }

    // Resample: walk the polygon perimeter and sample VERTS evenly-spaced points.
    let n = raw.len();
    let perimeter: f32 = (0..n).map(|i| raw[i].distance(raw[(i + 1) % n])).sum();
    let step = perimeter / VERTS as f32;

    let mut out = [Vec2::ZERO; VERTS];
    let mut seg = 0usize;
    let mut seg_t = 0.0f32;

    for v in out.iter_mut() {
        let a = raw[seg];
        let b = raw[(seg + 1) % n];
        let seg_len = a.distance(b);
        let local_t = if seg_len > 1e-6 { seg_t / seg_len } else { 0.0 };
        *v = a.lerp(b, local_t);

        seg_t += step;
        while seg_t >= raw[seg].distance(raw[(seg + 1) % n]) {
            seg_t -= raw[seg].distance(raw[(seg + 1) % n]);
            seg = (seg + 1) % n;
        }
    }

    out
}

fn blended_shape(contributions: &[(ScrapColor, ScrapShape, u16)], total: u32) -> Vec<Vec2> {
    // Accumulate per-shape weights (collapse across colors).
    let mut shape_weights = [0.0f32; 4];
    for (_, shape, count) in contributions {
        shape_weights[*shape as usize] += *count as f32 / total as f32;
    }

    // Weight-average the resampled vertex sets.
    let mut blended = [Vec2::ZERO; VERTS];
    for (shape_idx, &weight) in shape_weights.iter().enumerate() {
        if weight < 1e-6 {
            continue;
        }
        let shape = [
            ScrapShape::Diamond,
            ScrapShape::Circle,
            ScrapShape::Crescent,
            ScrapShape::Triangle,
        ][shape_idx];
        let verts = shape_verts(shape);
        for (b, v) in blended.iter_mut().zip(verts.iter()) {
            *b += *v * weight;
        }
    }

    // Scale to BASE_RADIUS and add per-vertex noise.
    let mut rng = rand::thread_rng();
    blended
        .iter()
        .map(|&v| {
            let noise = rng.gen_range(-NOISE_FRACTION..=NOISE_FRACTION);
            v * BASE_RADIUS * (1.0 + noise)
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Drawing
// ---------------------------------------------------------------------------

pub fn draw_mod_part(part: &ModPart, center: Vec2, driver: &mut dyn gfx::GraphicsDriver) {
    use gfx::{
        shape::polygon,
        style::{Fill, LineCap, LineJoin, Stroke, Style},
        tessellate, Color,
    };
    use glam::Mat3;

    let verts: Vec<Vec2> = part.shape.iter().map(|&v| center + v).collect();
    let [r, g, b] = part.avg_color;
    let style = Style {
        fill: Some(Fill::Solid(Color::rgba(r, g, b, 1.0))),
        stroke: Some(Stroke {
            color: Color::hex(0x000000FF),
            width: 0.008,
            cap: LineCap::Round,
            join: LineJoin::Round,
        }),
    };

    for mesh in tessellate(&polygon(&verts), &style) {
        let handle = driver.upload_mesh(&mesh.vertices, &mesh.indices);
        driver.draw_mesh(handle, Mat3::IDENTITY, [1.0, 1.0, 1.0, 1.0]);
    }
}
