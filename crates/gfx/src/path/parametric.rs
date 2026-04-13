use glam::Vec2;
use super::{Path, PathBuilder, Segment};

// ---------------------------------------------------------------------------
// Internal geometry representation
// ---------------------------------------------------------------------------

struct SegGeom {
    from:   Vec2,
    to:     Vec2,
    length: f32,
    kind:   SegKind,
}

enum SegKind {
    Line,
    Quad  { cp: Vec2 },
    Cubic { cp1: Vec2, cp2: Vec2 },
    Arc   { center: Vec2, radius: f32, start_angle: f32, end_angle: f32 },
}

/// Walk the segment list into drawable `SegGeom`s.
/// Move segments update the pen but produce no geometry entry.
/// If the path is closed, appends an implicit closing line when needed.
fn build_geoms(path: &Path) -> Vec<SegGeom> {
    let mut result = Vec::new();
    let mut pos    = Vec2::ZERO;
    let mut subpath_start = Vec2::ZERO;

    for seg in &path.segments {
        match seg {
            Segment::Move(p) => {
                pos           = *p;
                subpath_start = *p;
            }
            Segment::Line(end) => {
                let len = (*end - pos).length();
                result.push(SegGeom { from: pos, to: *end, length: len, kind: SegKind::Line });
                pos = *end;
            }
            Segment::Quad { cp, end } => {
                let len = quad_arc_length(pos, *cp, *end);
                result.push(SegGeom {
                    from: pos, to: *end, length: len,
                    kind: SegKind::Quad { cp: *cp },
                });
                pos = *end;
            }
            Segment::Cubic { cp1, cp2, end } => {
                let len = cubic_arc_length(pos, *cp1, *cp2, *end);
                result.push(SegGeom {
                    from: pos, to: *end, length: len,
                    kind: SegKind::Cubic { cp1: *cp1, cp2: *cp2 },
                });
                pos = *end;
            }
            Segment::Arc { center, radius, start_angle, end_angle } => {
                let arc_from = *center + Vec2::new(radius * start_angle.cos(), radius * start_angle.sin());
                let arc_to   = *center + Vec2::new(radius * end_angle.cos(),   radius * end_angle.sin());
                let len      = (end_angle - start_angle).abs() * radius;
                result.push(SegGeom {
                    from: arc_from, to: arc_to, length: len,
                    kind: SegKind::Arc {
                        center:      *center,
                        radius:      *radius,
                        start_angle: *start_angle,
                        end_angle:   *end_angle,
                    },
                });
                pos = arc_to;
            }
        }
    }

    // Implicit closing segment (line from last point back to subpath start).
    if path.closed && !result.is_empty() {
        let last = result.last().unwrap().to;
        if (last - subpath_start).length() > 1e-6 {
            let len = (last - subpath_start).length();
            result.push(SegGeom {
                from:   last,
                to:     subpath_start,
                length: len,
                kind:   SegKind::Line,
            });
        }
    }

    result
}

// ---------------------------------------------------------------------------
// Evaluate a point / tangent at segment-local t ∈ [0, 1]
// ---------------------------------------------------------------------------

fn eval_point(g: &SegGeom, t: f32) -> Vec2 {
    match &g.kind {
        SegKind::Line => g.from.lerp(g.to, t),
        SegKind::Quad { cp } => {
            let s = 1.0 - t;
            s * s * g.from + 2.0 * s * t * *cp + t * t * g.to
        }
        SegKind::Cubic { cp1, cp2 } => {
            let s = 1.0 - t;
            s*s*s * g.from
                + 3.0*s*s*t * *cp1
                + 3.0*s*t*t * *cp2
                + t*t*t * g.to
        }
        SegKind::Arc { center, radius, start_angle, end_angle } => {
            let angle = start_angle + t * (end_angle - start_angle);
            *center + Vec2::new(radius * angle.cos(), radius * angle.sin())
        }
    }
}

fn eval_tangent(g: &SegGeom, t: f32) -> Vec2 {
    match &g.kind {
        SegKind::Line => (g.to - g.from).normalize_or_zero(),
        SegKind::Quad { cp } => {
            let s = 1.0 - t;
            let d = 2.0 * (s * (*cp - g.from) + t * (g.to - *cp));
            d.normalize_or_zero()
        }
        SegKind::Cubic { cp1, cp2 } => {
            let s = 1.0 - t;
            let d = 3.0 * (s*s*(*cp1 - g.from)
                + 2.0*s*t*(*cp2 - *cp1)
                + t*t*(g.to - *cp2));
            d.normalize_or_zero()
        }
        SegKind::Arc { center: _, radius: _, start_angle, end_angle } => {
            let sweep = end_angle - start_angle;
            let angle = start_angle + t * sweep;
            let sign  = if sweep >= 0.0 { 1.0_f32 } else { -1.0_f32 };
            Vec2::new(-angle.sin() * sign, angle.cos() * sign)
        }
    }
}

/// Locate which segment and local-t correspond to arc-length target `s`.
fn find_seg(geoms: &[SegGeom], target: f32) -> (&SegGeom, f32) {
    let mut acc = 0.0_f32;
    for (i, g) in geoms.iter().enumerate() {
        let next = acc + g.length;
        if next >= target || i == geoms.len() - 1 {
            let lt = if g.length < 1e-10 { 0.0 }
                     else { ((target - acc) / g.length).clamp(0.0, 1.0) };
            return (g, lt);
        }
        acc = next;
    }
    (geoms.last().unwrap(), 1.0)
}

// ---------------------------------------------------------------------------
// 8-point Gauss-Legendre quadrature on [0, 1] for bezier arc lengths
// ---------------------------------------------------------------------------

const GL_NODES: [f32; 8] = [
    -0.960_289_856,
    -0.796_666_477,
    -0.525_532_409,
    -0.183_434_642,
     0.183_434_642,
     0.525_532_409,
     0.796_666_477,
     0.960_289_856,
];
const GL_WEIGHTS: [f32; 8] = [
    0.101_228_536,
    0.222_381_034,
    0.313_706_645,
    0.362_683_783,
    0.362_683_783,
    0.313_706_645,
    0.222_381_034,
    0.101_228_536,
];

fn integrate_speed(speed: impl Fn(f32) -> f32) -> f32 {
    GL_NODES.iter().zip(GL_WEIGHTS.iter())
        .map(|(&n, &w)| w * speed(0.5 * n + 0.5))
        .sum::<f32>()
        * 0.5
}

fn quad_arc_length(from: Vec2, cp: Vec2, to: Vec2) -> f32 {
    integrate_speed(|t| {
        let s = 1.0 - t;
        (2.0 * (s * (cp - from) + t * (to - cp))).length()
    })
}

fn cubic_arc_length(from: Vec2, cp1: Vec2, cp2: Vec2, to: Vec2) -> f32 {
    integrate_speed(|t| {
        let s = 1.0 - t;
        (3.0 * (s*s*(cp1 - from) + 2.0*s*t*(cp2 - cp1) + t*t*(to - cp2))).length()
    })
}

// ---------------------------------------------------------------------------
// De Casteljau split helpers
// ---------------------------------------------------------------------------

fn split_quad(from: Vec2, cp: Vec2, to: Vec2, t: f32)
    -> ((Vec2, Vec2, Vec2), (Vec2, Vec2, Vec2))
{
    let m01 = from.lerp(cp, t);
    let m12 = cp.lerp(to, t);
    let m   = m01.lerp(m12, t);
    ((from, m01, m), (m, m12, to))
}

fn split_cubic(from: Vec2, cp1: Vec2, cp2: Vec2, to: Vec2, t: f32)
    -> ((Vec2, Vec2, Vec2, Vec2), (Vec2, Vec2, Vec2, Vec2))
{
    let m01   = from.lerp(cp1, t);
    let m12   = cp1.lerp(cp2, t);
    let m23   = cp2.lerp(to, t);
    let m0112 = m01.lerp(m12, t);
    let m1222 = m12.lerp(m23, t);
    let m     = m0112.lerp(m1222, t);
    ((from, m01, m0112, m), (m, m1222, m23, to))
}

// ---------------------------------------------------------------------------
// PathBuilder append helpers (consume-and-return pattern)
// ---------------------------------------------------------------------------

fn append_full(b: PathBuilder, g: &SegGeom) -> PathBuilder {
    match &g.kind {
        SegKind::Line               => b.line_to(g.to),
        SegKind::Quad { cp }        => b.quad_to(*cp, g.to),
        SegKind::Cubic { cp1, cp2 } => b.cubic_to(*cp1, *cp2, g.to),
        SegKind::Arc { center, radius, start_angle, end_angle } =>
            b.arc_to(*center, *radius, *start_angle, *end_angle),
    }
}

/// Append the [0, local_t] portion of `g` to `b`.
fn append_first(b: PathBuilder, g: &SegGeom, local_t: f32, end_pt: Vec2) -> PathBuilder {
    match &g.kind {
        SegKind::Line => b.line_to(end_pt),
        SegKind::Quad { cp } => {
            let ((_, cp0, _), _) = split_quad(g.from, *cp, g.to, local_t);
            b.quad_to(cp0, end_pt)
        }
        SegKind::Cubic { cp1, cp2 } => {
            let ((_, c1, c2, _), _) = split_cubic(g.from, *cp1, *cp2, g.to, local_t);
            b.cubic_to(c1, c2, end_pt)
        }
        SegKind::Arc { center, radius, start_angle, end_angle } => {
            let mid = start_angle + local_t * (end_angle - start_angle);
            b.arc_to(*center, *radius, *start_angle, mid)
        }
    }
}

/// Append the [local_t, 1] portion of `g` to `b`.
fn append_last(b: PathBuilder, g: &SegGeom, local_t: f32) -> PathBuilder {
    match &g.kind {
        SegKind::Line => b.line_to(g.to),
        SegKind::Quad { cp } => {
            let (_, (_, cp1, end)) = split_quad(g.from, *cp, g.to, local_t);
            b.quad_to(cp1, end)
        }
        SegKind::Cubic { cp1, cp2 } => {
            let (_, (_, c1, c2, end)) = split_cubic(g.from, *cp1, *cp2, g.to, local_t);
            b.cubic_to(c1, c2, end)
        }
        SegKind::Arc { center, radius, start_angle, end_angle } => {
            let mid = start_angle + local_t * (end_angle - start_angle);
            b.arc_to(*center, *radius, mid, *end_angle)
        }
    }
}

// ---------------------------------------------------------------------------
// Path impl
// ---------------------------------------------------------------------------

impl Path {
    /// Total arc length of the path.
    pub fn length(&self) -> f32 {
        build_geoms(self).iter().map(|g| g.length).sum()
    }

    /// Point on the path at arc-length parameter `t ∈ [0, 1]`.
    pub fn point_at(&self, t: f32) -> Vec2 {
        let geoms = build_geoms(self);
        if geoms.is_empty() { return Vec2::ZERO; }

        let total: f32 = geoms.iter().map(|g| g.length).sum();
        if total <= 0.0 { return geoms[0].from; }

        let t = t.clamp(0.0, 1.0);
        if t >= 1.0 {
            // Closed path wraps back to start; open path ends at last point.
            return if self.closed { geoms[0].from } else { geoms.last().unwrap().to };
        }

        let (g, lt) = find_seg(&geoms, t * total);
        eval_point(g, lt)
    }

    /// Unit tangent vector at arc-length parameter `t`.
    pub fn tangent_at(&self, t: f32) -> Vec2 {
        let geoms = build_geoms(self);
        if geoms.is_empty() { return Vec2::X; }

        let total: f32 = geoms.iter().map(|g| g.length).sum();
        if total <= 0.0 { return Vec2::X; }

        // Clamp away from 1.0 to stay inside the last segment.
        let t = t.clamp(0.0, 1.0 - 1e-7);
        let (g, lt) = find_seg(&geoms, t * total);
        eval_tangent(g, lt)
    }

    /// Unit normal vector at `t` (left-perpendicular to tangent).
    pub fn normal_at(&self, t: f32) -> Vec2 {
        let tan = self.tangent_at(t);
        Vec2::new(-tan.y, tan.x)
    }

    /// Split the path into two at arc-length parameter `t`.
    pub fn split_at(&self, t: f32) -> (Path, Path) {
        let geoms = build_geoms(self);
        if geoms.is_empty() {
            return (self.clone(), self.clone());
        }

        let total: f32 = geoms.iter().map(|g| g.length).sum();
        if total <= 0.0 {
            return (self.clone(), self.clone());
        }

        let target  = (t.clamp(0.0, 1.0) * total).min(total);
        let mut acc = 0.0_f32;

        for (i, g) in geoms.iter().enumerate() {
            let next = acc + g.length;
            if next >= target || i == geoms.len() - 1 {
                let lt       = if g.length < 1e-10 { 0.0 }
                               else { ((target - acc) / g.length).clamp(0.0, 1.0) };
                let split_pt = eval_point(g, lt);

                // Prefix: move_to first point, then full segments up to i, then first
                // portion of segment i.
                let mut b1 = PathBuilder::new().move_to(geoms[0].from);
                for prev in &geoms[..i] {
                    b1 = append_full(b1, prev);
                }
                b1 = append_first(b1, g, lt, split_pt);
                let path1 = b1.build();

                // Suffix: start at split point, last portion of segment i, then
                // remaining full segments.
                let mut b2 = PathBuilder::new().move_to(split_pt);
                b2 = append_last(b2, g, lt);
                for rest in &geoms[(i + 1)..] {
                    b2 = append_full(b2, rest);
                }
                let path2 = if self.closed { b2.close() } else { b2.build() };

                return (path1, path2);
            }
            acc = next;
        }

        (self.clone(), Path { segments: vec![], closed: false })
    }

    /// Return the sub-path between arc-length parameters `t0` and `t1`.
    pub fn trim(&self, t0: f32, t1: f32) -> Path {
        let (_, tail) = self.split_at(t0);
        // `tail` spans [t0 .. 1.0]. We want up to t1, which is fraction
        // (t1 - t0) / (1 - t0) of tail's length.
        let span = 1.0 - t0;
        let local_t = if span < 1e-7 { 0.0 } else { ((t1 - t0) / span).clamp(0.0, 1.0) };
        let (trimmed, _) = tail.split_at(local_t);
        trimmed
    }

    /// Return the path traversed in the opposite direction.
    pub fn reverse(&self) -> Path {
        let geoms = build_geoms(self);
        if geoms.is_empty() { return self.clone(); }

        let start = geoms.last().unwrap().to;
        let mut b  = PathBuilder::new().move_to(start);

        for g in geoms.iter().rev() {
            b = match &g.kind {
                SegKind::Line               => b.line_to(g.from),
                SegKind::Quad { cp }        => b.quad_to(*cp, g.from),
                SegKind::Cubic { cp1, cp2 } => b.cubic_to(*cp2, *cp1, g.from),
                SegKind::Arc { center, radius, start_angle, end_angle } =>
                    b.arc_to(*center, *radius, *end_angle, *start_angle),
            };
        }

        if self.closed { b.close() } else { b.build() }
    }

    /// Axis-aligned bounding box of the path.
    pub fn bounding_box(&self) -> crate::scene::Rect {
        let geoms = build_geoms(self);
        if geoms.is_empty() {
            return crate::scene::Rect::new(Vec2::ZERO, Vec2::ZERO);
        }

        let mut min = Vec2::splat(f32::INFINITY);
        let mut max = Vec2::splat(f32::NEG_INFINITY);

        for g in &geoms {
            for &pt in &[g.from, g.to] {
                min = min.min(pt);
                max = max.max(pt);
            }
            // Coarse extra samples for non-linear segments.
            if !matches!(g.kind, SegKind::Line) {
                for k in 1..8u32 {
                    let p = eval_point(g, k as f32 / 8.0);
                    min = min.min(p);
                    max = max.max(p);
                }
            }
        }

        crate::scene::Rect::new(min, max - min)
    }

    /// Offset the path outward by `distance` units (parallel offset via dense sampling).
    pub fn offset(&self, distance: f32) -> Path {
        let geoms = build_geoms(self);
        if geoms.is_empty() { return self.clone(); }

        let total: f32 = geoms.iter().map(|g| g.length).sum();
        if total <= 0.0 { return self.clone(); }

        const N: usize = 64;
        let mut pts = Vec::with_capacity(N + 1);
        for i in 0..=N {
            // Keep t slightly below 1 to stay inside the last segment.
            let t = (i as f32 / N as f32).min(1.0 - 1e-7);
            let (g, lt) = find_seg(&geoms, t * total);
            let p   = eval_point(g, lt);
            let tan = eval_tangent(g, lt);
            let nor = Vec2::new(-tan.y, tan.x);
            pts.push(p + nor * distance);
        }

        let mut b = PathBuilder::new().move_to(pts[0]);
        for &pt in &pts[1..] {
            b = b.line_to(pt);
        }
        if self.closed { b.close() } else { b.build() }
    }

    /// Apply a transform to every control point in the path.
    pub fn transform(&self, t: crate::transform::Transform) -> Path {
        let segs = self.segments.iter().map(|seg| match seg {
            Segment::Move(p)                      => Segment::Move(t.apply(*p)),
            Segment::Line(p)                      => Segment::Line(t.apply(*p)),
            Segment::Quad { cp, end }             => Segment::Quad {
                cp:  t.apply(*cp),
                end: t.apply(*end),
            },
            Segment::Cubic { cp1, cp2, end }      => Segment::Cubic {
                cp1: t.apply(*cp1),
                cp2: t.apply(*cp2),
                end: t.apply(*end),
            },
            // Arc center is transformed; radius/angles assume uniform scale.
            Segment::Arc { center, radius, start_angle, end_angle } => Segment::Arc {
                center:      t.apply(*center),
                radius:      *radius,
                start_angle: *start_angle,
                end_angle:   *end_angle,
            },
        }).collect();
        Path { segments: segs, closed: self.closed }
    }
}
