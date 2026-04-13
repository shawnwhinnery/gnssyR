/// Internal tessellation: converts a (Path, Style) pair into triangle meshes
/// suitable for submission to a [`GraphicsDriver`].
///
/// Uses `lyon` for fill and stroke tessellation. This module is intentionally
/// `pub(crate)` — nothing outside `gfx` should depend on lyon directly.
use std::f32::consts::PI;

use lyon::math::point;
use lyon::path::Path as LyonPath;
use lyon::tessellation::{
    BuffersBuilder, FillOptions, FillTessellator, FillVertex, FillVertexConstructor,
    StrokeOptions, StrokeTessellator, StrokeVertex, StrokeVertexConstructor, VertexBuffers,
};

use crate::{
    driver::{Mesh, Vertex},
    style::{Fill, LineCap, LineJoin, Stroke, Style},
};
use super::{Path, Segment};

// ---------------------------------------------------------------------------
// Public(crate) entry point
// ---------------------------------------------------------------------------

pub(crate) fn tessellate(path: &Path, style: &Style) -> Vec<Mesh> {
    let mut meshes = Vec::new();

    if let Some(fill) = &style.fill {
        if let Some(mesh) = tessellate_fill(path, fill) {
            if !mesh.vertices.is_empty() {
                meshes.push(mesh);
            }
        }
    }

    if let Some(stroke) = &style.stroke {
        if let Some(mesh) = tessellate_stroke(path, stroke) {
            if !mesh.vertices.is_empty() {
                meshes.push(mesh);
            }
        }
    }

    meshes
}

// ---------------------------------------------------------------------------
// Path → lyon path conversion
// ---------------------------------------------------------------------------

fn to_lyon_path(path: &Path) -> Option<LyonPath> {
    if path.segments.is_empty() {
        return None;
    }

    let mut b = LyonPath::builder();
    let mut begun = false;

    for seg in &path.segments {
        match seg {
            Segment::Move(p) => {
                if begun {
                    // End the previous subpath (open); a new one starts here.
                    b.end(false);
                }
                b.begin(point(p.x, p.y));
                begun = true;
            }
            Segment::Line(end) => {
                if !begun {
                    b.begin(point(0.0, 0.0));
                    begun = true;
                }
                b.line_to(point(end.x, end.y));
            }
            Segment::Quad { cp, end } => {
                if !begun {
                    b.begin(point(0.0, 0.0));
                    begun = true;
                }
                b.quadratic_bezier_to(point(cp.x, cp.y), point(end.x, end.y));
            }
            Segment::Cubic { cp1, cp2, end } => {
                if !begun {
                    b.begin(point(0.0, 0.0));
                    begun = true;
                }
                b.cubic_bezier_to(
                    point(cp1.x, cp1.y),
                    point(cp2.x, cp2.y),
                    point(end.x, end.y),
                );
            }
            Segment::Arc { center, radius, start_angle, end_angle } => {
                // Approximate arc with line segments (~11.25° each).
                let sx = center.x + radius * start_angle.cos();
                let sy = center.y + radius * start_angle.sin();
                if !begun {
                    b.begin(point(sx, sy));
                    begun = true;
                } else {
                    // Bridge gap from previous endpoint to arc start.
                    b.line_to(point(sx, sy));
                }
                let sweep = end_angle - start_angle;
                let n = ((sweep.abs() / (PI / 16.0)).ceil() as u32).max(2);
                for i in 1..=n {
                    let a = start_angle + sweep * (i as f32 / n as f32);
                    b.line_to(point(
                        center.x + radius * a.cos(),
                        center.y + radius * a.sin(),
                    ));
                }
            }
        }
    }

    if begun {
        b.end(path.closed);
        Some(b.build())
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Vertex constructors
// ---------------------------------------------------------------------------

struct FillCtor([f32; 4]);
impl FillVertexConstructor<Vertex> for FillCtor {
    fn new_vertex(&mut self, v: FillVertex<'_>) -> Vertex {
        let p = v.position();
        Vertex { position: [p.x, p.y], color: self.0 }
    }
}

struct StrokeCtor([f32; 4]);
impl StrokeVertexConstructor<Vertex> for StrokeCtor {
    fn new_vertex(&mut self, v: StrokeVertex<'_, '_>) -> Vertex {
        let p = v.position();
        Vertex { position: [p.x, p.y], color: self.0 }
    }
}

// ---------------------------------------------------------------------------
// Fill tessellation
// ---------------------------------------------------------------------------

fn tessellate_fill(path: &Path, fill: &Fill) -> Option<Mesh> {
    let color = fill_color(fill);
    let lyon_path = to_lyon_path(path)?;

    let mut buffers: VertexBuffers<Vertex, u32> = VertexBuffers::new();
    FillTessellator::new()
        .tessellate_path(
            &lyon_path,
            &FillOptions::default().with_tolerance(0.001),
            &mut BuffersBuilder::new(&mut buffers, FillCtor(color)),
        )
        .ok()?;

    Some(Mesh { vertices: buffers.vertices, indices: buffers.indices })
}

// ---------------------------------------------------------------------------
// Stroke tessellation
// ---------------------------------------------------------------------------

fn tessellate_stroke(path: &Path, stroke: &Stroke) -> Option<Mesh> {
    let color = stroke.color.to_array();
    let lyon_path = to_lyon_path(path)?;

    let lyon_cap = match stroke.cap {
        LineCap::Butt   => lyon::tessellation::LineCap::Butt,
        LineCap::Round  => lyon::tessellation::LineCap::Round,
        LineCap::Square => lyon::tessellation::LineCap::Square,
    };
    let lyon_join = match stroke.join {
        LineJoin::Miter => lyon::tessellation::LineJoin::Miter,
        LineJoin::Round => lyon::tessellation::LineJoin::Round,
        LineJoin::Bevel => lyon::tessellation::LineJoin::Bevel,
    };

    let options = StrokeOptions::default()
        .with_tolerance(0.001)
        .with_line_width(stroke.width)
        .with_start_cap(lyon_cap)
        .with_end_cap(lyon_cap)
        .with_line_join(lyon_join);

    let mut buffers: VertexBuffers<Vertex, u32> = VertexBuffers::new();
    StrokeTessellator::new()
        .tessellate_path(
            &lyon_path,
            &options,
            &mut BuffersBuilder::new(&mut buffers, StrokeCtor(color)),
        )
        .ok()?;

    Some(Mesh { vertices: buffers.vertices, indices: buffers.indices })
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn fill_color(fill: &Fill) -> [f32; 4] {
    match fill {
        Fill::Solid(c) => c.to_array(),
        Fill::LinearGradient { stops, .. } | Fill::RadialGradient { stops, .. } => {
            stops.first().map(|s| s.color.to_array()).unwrap_or([1.0; 4])
        }
    }
}
