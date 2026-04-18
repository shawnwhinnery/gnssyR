use std::f32::consts::{PI, TAU};

use gfx::{
    path::PathBuilder,
    shape::{circle, ellipse, line, polygon, polyline, rect, regular_polygon, rounded_rect, star},
    style::{LineCap, Stroke, Style},
    tessellate, Color, Transform, Vec2,
};

/// Render the full GFX showcase scene onto `driver`.
///
/// Exercises every shape primitive, style variant, and transform helper in the
/// vector library. Suitable for use in both the live binary and headless
/// snapshot tests.
///
/// Assumes `begin_frame` has already been called; does not call `end_frame`.
pub fn draw_scene(driver: &mut dyn gfx::GraphicsDriver) {
    driver.clear([0.04, 0.04, 0.10, 1.0]);

    // ----------------------------------------------------------------
    // Row 1 — top  (y ≈ +0.62)
    // ----------------------------------------------------------------

    // Star — 5 points, filled gold, rotated PI/10 around its centre.
    draw_shape(
        driver,
        &star(Vec2::new(-0.62, 0.62), 0.18, 0.08, 5),
        &Style::filled(Color::hex(0xFFD700FF)),
        rotate_around(-0.62, 0.62, PI / 10.0),
    );

    // Regular hexagon — purple fill + thin white stroke.
    draw_shape(
        driver,
        &regular_polygon(Vec2::new(0.0, 0.65), 0.16, 6),
        &Style {
            fill: Some(gfx::style::Fill::Solid(Color::hex(0x9B59B6FF))),
            stroke: Some(Stroke::solid(Color::WHITE, 0.006)),
        },
        glam::Mat3::IDENTITY,
    );

    // Ellipse — teal fill + white stroke, slightly squashed vertically.
    draw_shape(
        driver,
        &ellipse(Vec2::new(0.62, 0.62), 0.20, 0.11),
        &Style {
            fill: Some(gfx::style::Fill::Solid(Color::hex(0x2EC4B6FF))),
            stroke: Some(Stroke::solid(Color::hex(0xEEEEEEFF), 0.005)),
        },
        glam::Mat3::IDENTITY,
    );

    // ----------------------------------------------------------------
    // Row 2 — middle  (y ≈ 0.0)
    // ----------------------------------------------------------------

    // Circle — coral fill + dark stroke, rotated (no-op but exercises the path).
    draw_shape(
        driver,
        &circle(Vec2::new(-0.62, 0.0), 0.15),
        &Style {
            fill: Some(gfx::style::Fill::Solid(Color::hex(0xFF6B6BFF))),
            stroke: Some(Stroke {
                color: Color::hex(0x220000FF),
                width: 0.007,
                cap: LineCap::Round,
                join: gfx::style::LineJoin::Round,
            }),
        },
        glam::Mat3::IDENTITY,
    );

    // Rounded rect — sky-blue fill + white stroke, centred.
    draw_shape(
        driver,
        &rounded_rect(Vec2::new(-0.18, -0.13), Vec2::new(0.36, 0.26), 0.04),
        &Style {
            fill: Some(gfx::style::Fill::Solid(Color::hex(0x5DADE2FF))),
            stroke: Some(Stroke::solid(Color::WHITE, 0.006)),
        },
        glam::Mat3::IDENTITY,
    );

    // Triangle polygon — orange fill.
    draw_shape(
        driver,
        &polygon(&[
            Vec2::new(0.62, 0.14),
            Vec2::new(0.47, -0.14),
            Vec2::new(0.77, -0.14),
        ]),
        &Style::filled(Color::hex(0xFF9500FF)),
        glam::Mat3::IDENTITY,
    );

    // ----------------------------------------------------------------
    // Row 3 — bottom  (y ≈ -0.62)
    // ----------------------------------------------------------------

    // Plain rect — stroke only (white), scaled tall via transform.
    {
        let r = rect(Vec2::new(-0.78, -0.78), Vec2::new(0.28, 0.22));
        let scale_t = Transform::translate(-0.64, -0.67)
            .then(Transform::scale(1.0, 1.4))
            .then(Transform::translate(0.64, 0.67));
        draw_shape(
            driver,
            &r,
            &Style::stroked(Stroke::solid(Color::WHITE, 0.006)),
            scale_t.to_mat3(),
        );
    }

    // Polyline — yellow zigzag stroke.
    draw_shape(
        driver,
        &polyline(&[
            Vec2::new(-0.22, -0.55),
            Vec2::new(-0.10, -0.72),
            Vec2::new(0.02, -0.55),
            Vec2::new(0.14, -0.72),
            Vec2::new(0.26, -0.55),
        ]),
        &Style::stroked(Stroke::solid(Color::hex(0xF1C40FFF), 0.008)),
        glam::Mat3::IDENTITY,
    );

    // Custom path — cubic bezier + arc, blue fill.
    {
        let path = PathBuilder::new()
            .move_to(Vec2::new(0.46, -0.52))
            .cubic_to(
                Vec2::new(0.58, -0.52),
                Vec2::new(0.78, -0.60),
                Vec2::new(0.78, -0.72),
            )
            .arc_to(Vec2::new(0.62, -0.72), 0.16, 0.0, PI)
            .close();
        draw_shape(
            driver,
            &path,
            &Style {
                fill: Some(gfx::style::Fill::Solid(Color::hex(0x3498DBFF))),
                stroke: Some(Stroke::solid(Color::hex(0xECF0F1FF), 0.005)),
            },
            glam::Mat3::IDENTITY,
        );
    }

    // ----------------------------------------------------------------
    // Decorative lines across the scene
    // ----------------------------------------------------------------

    // Diagonal white line (exercises line() + LineCap::Square stroke).
    draw_shape(
        driver,
        &line(Vec2::new(-0.95, -0.95), Vec2::new(-0.30, -0.30)),
        &Style::stroked(Stroke {
            color: Color::rgba(1.0, 1.0, 1.0, 0.35),
            width: 0.004,
            cap: LineCap::Square,
            join: gfx::style::LineJoin::Miter,
        }),
        glam::Mat3::IDENTITY,
    );

    // Horizontal line through centre — exercises Color::with_alpha.
    draw_shape(
        driver,
        &line(Vec2::new(-0.90, 0.0), Vec2::new(0.90, 0.0)),
        &Style::stroked(Stroke::solid(Color::WHITE.with_alpha(0.12), 0.003)),
        glam::Mat3::IDENTITY,
    );

    // Vertical line — exercises Color::to_array indirectly via tessellate.
    draw_shape(
        driver,
        &line(Vec2::new(0.0, -0.90), Vec2::new(0.0, 0.90)),
        &Style::stroked(Stroke::solid(Color::WHITE.with_alpha(0.12), 0.003)),
        glam::Mat3::IDENTITY,
    );

    // ----------------------------------------------------------------
    // Parametric path: sample a quadratic-bezier path and draw dots
    // along it using offset() to create a parallel curve.
    // ----------------------------------------------------------------
    {
        let base = PathBuilder::new()
            .move_to(Vec2::new(-0.90, 0.32))
            .quad_to(Vec2::new(-0.55, 0.90), Vec2::new(-0.20, 0.32))
            .build();

        // Draw the base curve as a stroke.
        draw_shape(
            driver,
            &base,
            &Style::stroked(Stroke::solid(Color::hex(0xE74C3CFF), 0.005)),
            glam::Mat3::IDENTITY,
        );

        // Offset parallel curve.
        let offset_curve = base.offset(0.06);
        draw_shape(
            driver,
            &offset_curve,
            &Style::stroked(Stroke::solid(Color::hex(0xE74C3CFF).with_alpha(0.5), 0.003)),
            glam::Mat3::IDENTITY,
        );
    }

    // ----------------------------------------------------------------
    // Transform showcase: eight small circles orbiting (0, 0.33).
    // ----------------------------------------------------------------
    for i in 0..8u32 {
        let angle = TAU * i as f32 / 8.0;
        let t = Transform::translate(0.0, 0.33)
            .then(Transform::rotate(angle))
            .then(Transform::translate(0.0, -0.33));
        let c = circle(Vec2::new(0.0, 0.33), 0.025);
        let hue = (i as f32 / 8.0 * 360.0) as u32;
        draw_shape(
            driver,
            &c,
            &Style::filled(hsv_color(hue, 0.8, 1.0)),
            t.to_mat3(),
        );
    }
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

fn draw_shape(
    driver: &mut dyn gfx::GraphicsDriver,
    path: &gfx::Path,
    style: &gfx::Style,
    transform: glam::Mat3,
) {
    for mesh in tessellate(path, style) {
        let handle = driver.upload_mesh(&mesh.vertices, &mesh.indices);
        driver.draw_mesh(handle, transform, [1.0, 1.0, 1.0, 1.0]);
    }
}

fn rotate_around(cx: f32, cy: f32, angle: f32) -> glam::Mat3 {
    Transform::translate(-cx, -cy)
        .then(Transform::rotate(angle))
        .then(Transform::translate(cx, cy))
        .to_mat3()
}

fn hsv_color(h_deg: u32, s: f32, v: f32) -> Color {
    let h = h_deg as f32 / 60.0;
    let i = h as u32;
    let f = h - i as f32;
    let p = v * (1.0 - s);
    let q = v * (1.0 - s * f);
    let t = v * (1.0 - s * (1.0 - f));
    let (r, g, b) = match i % 6 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };
    Color::rgba(r, g, b, 1.0)
}
