use glam::{Mat3, Vec2};

use crate::camera::Camera;
use gfx::{
    path::PathBuilder,
    shape::{circle, polygon},
    style::{Fill, LineCap, LineJoin, Stroke, Style},
    tessellate, Color, Path,
};

const SCRAP_RADIUS: f32 = 0.18;
const NUM_COLORS: usize = 8;
const NUM_SHAPES: usize = 4;
const NUM_KINDS: usize = NUM_COLORS * NUM_SHAPES;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ScrapColor {
    Red,
    Orange,
    Yellow,
    Green,
    Cyan,
    Blue,
    Purple,
    Pink,
}

impl ScrapColor {
    pub const ALL: [ScrapColor; NUM_COLORS] = [
        ScrapColor::Red,
        ScrapColor::Orange,
        ScrapColor::Yellow,
        ScrapColor::Green,
        ScrapColor::Cyan,
        ScrapColor::Blue,
        ScrapColor::Purple,
        ScrapColor::Pink,
    ];

    fn to_gfx(self) -> Color {
        match self {
            ScrapColor::Red => Color::rgba(0.90, 0.15, 0.15, 1.0),
            ScrapColor::Orange => Color::rgba(0.95, 0.50, 0.10, 1.0),
            ScrapColor::Yellow => Color::rgba(0.95, 0.90, 0.10, 1.0),
            ScrapColor::Green => Color::rgba(0.10, 0.85, 0.20, 1.0),
            ScrapColor::Cyan => Color::rgba(0.10, 0.85, 0.90, 1.0),
            ScrapColor::Blue => Color::rgba(0.15, 0.30, 0.95, 1.0),
            ScrapColor::Purple => Color::rgba(0.65, 0.10, 0.90, 1.0),
            ScrapColor::Pink => Color::rgba(0.95, 0.35, 0.75, 1.0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ScrapShape {
    Diamond,
    Circle,
    Crescent,
    Triangle,
}

impl ScrapShape {
    pub const ALL: [ScrapShape; NUM_SHAPES] = [
        ScrapShape::Diamond,
        ScrapShape::Circle,
        ScrapShape::Crescent,
        ScrapShape::Triangle,
    ];
}

/// Crescent polygon vertices centered at the origin with the given radius.
///
/// Samples the same two-circle construction used by `crescent_path`, returned
/// as a flat vertex list for use in polygon-blending (e.g. mod_part).
pub fn crescent_verts(r: f32) -> Vec<Vec2> {
    const INNER_SCALE: f32 = 0.90;
    const INNER_SHIFT: f32 = 0.55;
    const SEGS: usize = 20;

    let inner_r = r * INNER_SCALE;
    let inner_cx = r * INNER_SHIFT;

    let xi = (r * r - inner_r * inner_r + inner_cx * inner_cx) / (2.0 * inner_cx);
    let yi = (r * r - xi * xi).max(0.0).sqrt();

    let outer_angle = yi.atan2(xi);
    let inner_angle = yi.atan2(xi - inner_cx);
    let outer_sweep = std::f32::consts::TAU - 2.0 * outer_angle;

    let mut pts = Vec::with_capacity(2 * SEGS);

    for i in 0..=SEGS {
        let t = i as f32 / SEGS as f32;
        let a = outer_angle + outer_sweep * t;
        pts.push(Vec2::new(a.cos() * r, a.sin() * r));
    }

    // CW from -inner_angle through ±180° to +inner_angle (the concave left face).
    let inner_sweep = -(std::f32::consts::TAU - 2.0 * inner_angle);
    for i in 1..SEGS {
        let t = i as f32 / SEGS as f32;
        let a = -inner_angle + inner_sweep * t;
        pts.push(Vec2::new(inner_cx + a.cos() * inner_r, a.sin() * inner_r));
    }

    pts
}

/// Crescent path centered at `center` with the given radius.
///
/// Two-circle construction:
///   Outer circle: radius `r`, centered at `center` (provides the convex back)
///   Inner circle: radius `INNER_SCALE * r`, centered at `center + (INNER_SHIFT * r, 0)`
///     (cuts the concave bite)
/// The crescent body is on the left; the concave opening faces right.
/// Uses `arc_to` segments so the tessellator produces smooth curves automatically.
pub fn crescent_path(center: Vec2, r: f32) -> Path {
    const INNER_SCALE: f32 = 0.90;
    const INNER_SHIFT: f32 = 0.55;

    let inner_r = r * INNER_SCALE;
    let inner_cx = r * INNER_SHIFT;
    let inner_center = center + Vec2::new(inner_cx, 0.0);

    // Intersection of the two circles.
    let xi = (r * r - inner_r * inner_r + inner_cx * inner_cx) / (2.0 * inner_cx);
    let yi = (r * r - xi * xi).max(0.0).sqrt();

    // Angles on the outer circle at the intersection points.
    let outer_angle = yi.atan2(xi - 0.0); // relative to outer center (origin)
                                          // Angles on the inner circle at the intersection points.
    let inner_angle = yi.atan2(xi - inner_cx); // may be obtuse when xi < inner_cx

    // Outer arc: from top intersection CCW around the left/back to bottom intersection.
    //   start = +outer_angle, end = TAU - outer_angle  (CCW, long way around)
    // Inner arc: from bottom intersection CW through ±180° to top intersection.
    //   Uses a negative sweep so the arc traces the LEFT (concave) side of the
    //   inner circle — the face that forms the hollow of the crescent — rather
    //   than the right side, which would bulge outside the outer circle.
    PathBuilder::new()
        .arc_to(center, r, outer_angle, std::f32::consts::TAU - outer_angle)
        .arc_to(
            inner_center,
            inner_r,
            -inner_angle,
            inner_angle - std::f32::consts::TAU,
        )
        .close()
}

// ---------------------------------------------------------------------------
// In-world scrap entity
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
pub struct Scrap {
    pub position: Vec2,
    pub color: ScrapColor,
    pub shape: ScrapShape,
}

// ---------------------------------------------------------------------------
// Inventory
// ---------------------------------------------------------------------------

/// Fixed-size counter table: one u16 per (color × shape) permutation.
/// Total size: 8 colors × 4 shapes × 2 bytes = 64 bytes.
#[derive(Debug, Clone)]
pub struct Inventory {
    counts: [u16; NUM_KINDS],
}

impl Inventory {
    pub fn new() -> Self {
        Self {
            counts: [0; NUM_KINDS],
        }
    }

    pub fn add(&mut self, color: ScrapColor, shape: ScrapShape) {
        let i = color as usize * NUM_SHAPES + shape as usize;
        self.counts[i] = self.counts[i].saturating_add(1);
    }

    pub fn count(&self, color: ScrapColor, shape: ScrapShape) -> u16 {
        self.counts[color as usize * NUM_SHAPES + shape as usize]
    }

    pub fn remove(&mut self, color: ScrapColor, shape: ScrapShape, amount: u16) {
        let i = color as usize * NUM_SHAPES + shape as usize;
        self.counts[i] = self.counts[i].saturating_sub(amount);
    }

    pub fn count_shape(&self, shape: ScrapShape) -> u32 {
        ScrapColor::ALL
            .iter()
            .map(|&color| self.count(color, shape) as u32)
            .sum()
    }

    pub fn total(&self) -> u32 {
        self.counts.iter().map(|&c| c as u32).sum()
    }
}

impl Default for Inventory {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------

pub fn draw_scrap(scrap: &Scrap, driver: &mut dyn gfx::GraphicsDriver, camera: &Camera) {
    let ndc = camera.world_to_ndc(scrap.position);
    let r = camera.scale(SCRAP_RADIUS);

    let style = Style {
        fill: Some(Fill::Solid(scrap.color.to_gfx())),
        stroke: Some(Stroke {
            color: Color::hex(0x000000CC),
            width: 0.005,
            cap: LineCap::Round,
            join: LineJoin::Round,
        }),
    };

    let path = match scrap.shape {
        ScrapShape::Diamond => {
            let verts = [
                ndc + Vec2::new(0.0, r),
                ndc + Vec2::new(r, 0.0),
                ndc + Vec2::new(0.0, -r),
                ndc + Vec2::new(-r, 0.0),
            ];
            polygon(&verts)
        }
        ScrapShape::Circle => circle(ndc, r),
        ScrapShape::Crescent => crescent_path(ndc, r),
        ScrapShape::Triangle => {
            let verts = [
                ndc + Vec2::new(0.0, r),
                ndc + Vec2::new(-r * 0.866, -r * 0.5),
                ndc + Vec2::new(r * 0.866, -r * 0.5),
            ];
            polygon(&verts)
        }
    };

    for mesh in tessellate(&path, &style) {
        let handle = driver.upload_mesh(&mesh.vertices, &mesh.indices);
        driver.draw_mesh(handle, Mat3::IDENTITY, [1.0, 1.0, 1.0, 1.0]);
    }
}
