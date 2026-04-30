use glam::{Mat3, Vec2};

use crate::camera::Camera;
use gfx::{
    shape::{circle, polygon},
    style::{Fill, LineCap, LineJoin, Stroke, Style},
    tessellate, Color,
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
            ScrapColor::Red    => Color::rgba(0.90, 0.15, 0.15, 1.0),
            ScrapColor::Orange => Color::rgba(0.95, 0.50, 0.10, 1.0),
            ScrapColor::Yellow => Color::rgba(0.95, 0.90, 0.10, 1.0),
            ScrapColor::Green  => Color::rgba(0.10, 0.85, 0.20, 1.0),
            ScrapColor::Cyan   => Color::rgba(0.10, 0.85, 0.90, 1.0),
            ScrapColor::Blue   => Color::rgba(0.15, 0.30, 0.95, 1.0),
            ScrapColor::Purple => Color::rgba(0.65, 0.10, 0.90, 1.0),
            ScrapColor::Pink   => Color::rgba(0.95, 0.35, 0.75, 1.0),
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
    pub const ALL: [ScrapShape; NUM_SHAPES] =
        [ScrapShape::Diamond, ScrapShape::Circle, ScrapShape::Crescent, ScrapShape::Triangle];
}

/// Crescent polygon vertices centered at the origin with the given radius.
///
/// Two-circle construction:
///   Outer circle: radius `r`, centered at origin (provides the convex back)
///   Inner circle: radius `0.75 * r`, centered at `(0.35 * r, 0)` (cuts the concave bite)
/// The crescent body is on the left; the concave opening faces right.
pub fn crescent_verts(r: f32) -> Vec<Vec2> {
    const INNER_SCALE: f32 = 0.75;
    const INNER_SHIFT: f32 = 0.35;
    const SEGS: usize = 10;

    let inner_r = r * INNER_SCALE;
    let inner_cx = r * INNER_SHIFT;

    // x-coordinate of the intersection of the two circles
    let xi = (r * r - inner_r * inner_r + inner_cx * inner_cx) / (2.0 * inner_cx);
    let yi = (r * r - xi * xi).max(0.0).sqrt();

    let outer_angle = yi.atan2(xi);
    let inner_angle = yi.atan2(xi - inner_cx);
    let outer_sweep = std::f32::consts::TAU - 2.0 * outer_angle;

    let mut pts = Vec::with_capacity(2 * SEGS);

    // Outer arc: +outer_angle → CCW sweep → -outer_angle  (the convex left side)
    for i in 0..=SEGS {
        let t = i as f32 / SEGS as f32;
        let a = outer_angle + outer_sweep * t;
        pts.push(Vec2::new(a.cos() * r, a.sin() * r));
    }

    // Inner arc: -inner_angle → CCW → +inner_angle  (the concave right side)
    // Skip endpoints — they duplicate the outer arc's start/end.
    for i in 1..SEGS {
        let t = i as f32 / SEGS as f32;
        let a = -inner_angle + 2.0 * inner_angle * t;
        pts.push(Vec2::new(inner_cx + a.cos() * inner_r, a.sin() * inner_r));
    }

    pts
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
        Self { counts: [0; NUM_KINDS] }
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
        ScrapShape::Crescent => {
            let raw = crescent_verts(r);
            let verts: Vec<Vec2> = raw.iter().map(|v| ndc + *v).collect();
            polygon(&verts)
        }
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
