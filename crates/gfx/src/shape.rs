/// Shape primitive constructors — all return a [`Path`].
///
/// None of these carry style or transform; they are pure geometry.
/// Apply a [`Style`] and [`Transform`] when adding to a [`Scene`].
use glam::Vec2;
use crate::path::{Path, PathBuilder};
use std::f32::consts::TAU;

pub fn circle(center: Vec2, radius: f32) -> Path {
    // Approximate circle with a cubic bezier arc (4-segment approximation).
    let k = 0.5522847498;  // magic constant for circle → bezier
    let r = radius;
    PathBuilder::new()
        .move_to(center + Vec2::new(r, 0.0))
        .cubic_to(
            center + Vec2::new(r,     r * k),
            center + Vec2::new(r * k, r),
            center + Vec2::new(0.0,   r),
        )
        .cubic_to(
            center + Vec2::new(-r * k, r),
            center + Vec2::new(-r,     r * k),
            center + Vec2::new(-r,     0.0),
        )
        .cubic_to(
            center + Vec2::new(-r,    -r * k),
            center + Vec2::new(-r * k, -r),
            center + Vec2::new(0.0,   -r),
        )
        .cubic_to(
            center + Vec2::new(r * k, -r),
            center + Vec2::new(r,    -r * k),
            center + Vec2::new(r,     0.0),
        )
        .close()
}

pub fn ellipse(center: Vec2, rx: f32, ry: f32) -> Path {
    let k = 0.5522847498;
    PathBuilder::new()
        .move_to(center + Vec2::new(rx, 0.0))
        .cubic_to(
            center + Vec2::new(rx,      ry * k),
            center + Vec2::new(rx * k,  ry),
            center + Vec2::new(0.0,     ry),
        )
        .cubic_to(
            center + Vec2::new(-rx * k, ry),
            center + Vec2::new(-rx,     ry * k),
            center + Vec2::new(-rx,     0.0),
        )
        .cubic_to(
            center + Vec2::new(-rx,    -ry * k),
            center + Vec2::new(-rx * k, -ry),
            center + Vec2::new(0.0,    -ry),
        )
        .cubic_to(
            center + Vec2::new(rx * k, -ry),
            center + Vec2::new(rx,    -ry * k),
            center + Vec2::new(rx,     0.0),
        )
        .close()
}

pub fn rect(origin: Vec2, size: Vec2) -> Path {
    PathBuilder::new()
        .move_to(origin)
        .line_to(origin + Vec2::new(size.x, 0.0))
        .line_to(origin + size)
        .line_to(origin + Vec2::new(0.0, size.y))
        .close()
}

pub fn rounded_rect(origin: Vec2, size: Vec2, r: f32) -> Path {
    let r = r.min(size.x / 2.0).min(size.y / 2.0);
    let k = 0.5522847498 * r;
    let x0 = origin.x;
    let y0 = origin.y;
    let x1 = origin.x + size.x;
    let y1 = origin.y + size.y;

    PathBuilder::new()
        .move_to(Vec2::new(x0 + r, y0))
        .line_to(Vec2::new(x1 - r, y0))
        .cubic_to(Vec2::new(x1 - r + k, y0), Vec2::new(x1, y0 + r - k), Vec2::new(x1, y0 + r))
        .line_to(Vec2::new(x1, y1 - r))
        .cubic_to(Vec2::new(x1, y1 - r + k), Vec2::new(x1 - r + k, y1), Vec2::new(x1 - r, y1))
        .line_to(Vec2::new(x0 + r, y1))
        .cubic_to(Vec2::new(x0 + r - k, y1), Vec2::new(x0, y1 - r + k), Vec2::new(x0, y1 - r))
        .line_to(Vec2::new(x0, y0 + r))
        .cubic_to(Vec2::new(x0, y0 + r - k), Vec2::new(x0 + r - k, y0), Vec2::new(x0 + r, y0))
        .close()
}

pub fn regular_polygon(center: Vec2, radius: f32, sides: u32) -> Path {
    assert!(sides >= 3, "polygon needs at least 3 sides");
    let mut b = PathBuilder::new();
    for i in 0..sides {
        let angle = TAU * i as f32 / sides as f32 - std::f32::consts::FRAC_PI_2;
        let p = center + Vec2::new(angle.cos(), angle.sin()) * radius;
        b = if i == 0 { b.move_to(p) } else { b.line_to(p) };
    }
    b.close()
}

pub fn star(center: Vec2, outer: f32, inner: f32, points: u32) -> Path {
    assert!(points >= 2, "star needs at least 2 points");
    let total = points * 2;
    let mut b = PathBuilder::new();
    for i in 0..total {
        let angle = TAU * i as f32 / total as f32 - std::f32::consts::FRAC_PI_2;
        let r = if i % 2 == 0 { outer } else { inner };
        let p = center + Vec2::new(angle.cos(), angle.sin()) * r;
        b = if i == 0 { b.move_to(p) } else { b.line_to(p) };
    }
    b.close()
}

pub fn line(start: Vec2, end: Vec2) -> Path {
    PathBuilder::new().move_to(start).line_to(end).build()
}

pub fn polyline(points: &[Vec2]) -> Path {
    assert!(!points.is_empty(), "polyline needs at least one point");
    let mut b = PathBuilder::new().move_to(points[0]);
    for &p in &points[1..] {
        b = b.line_to(p);
    }
    b.build()
}

pub fn polygon(points: &[Vec2]) -> Path {
    assert!(points.len() >= 3, "polygon needs at least 3 points");
    let mut b = PathBuilder::new().move_to(points[0]);
    for &p in &points[1..] {
        b = b.line_to(p);
    }
    b.close()
}
