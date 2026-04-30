use std::f32::consts::{PI, TAU};

use gfx::color::Color;
use gfx::path::{Path, PathBuilder};
use gfx::scene::{Group, Node, Scene, Shape};
use gfx::shape::{
    circle, ellipse, line, polygon, polyline, rect, regular_polygon, rounded_rect, star,
};
use gfx::style::{LineCap, LineJoin, Stroke, Style};
use gfx::transform::Transform;
use glam::{Mat3, Vec2};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn approx(a: f32, b: f32, eps: f32) -> bool {
    (a - b).abs() < eps
}
fn approx_vec(a: Vec2, b: Vec2, eps: f32) -> bool {
    (a - b).length() < eps
}

// A SpyDriver that records which GraphicsDriver methods were called.
use gfx::driver::{GraphicsDriver, MeshHandle, TextureHandle, Vertex};
#[derive(Default)]
struct SpyDriver {
    begin_frames: u32,
    end_frames: u32,
    upload_calls: u32,
    draw_calls: u32,
    upload_texture_calls: u32,
    free_texture_calls: u32,
    draw_bitmap_calls: u32,
    last_transform: Option<Mat3>,
}
impl GraphicsDriver for SpyDriver {
    fn begin_frame(&mut self) {
        self.begin_frames += 1;
    }
    fn end_frame(&mut self) {
        self.end_frames += 1;
    }
    fn present(&mut self) {}
    fn clear(&mut self, _: [f32; 4]) {}
    fn upload_mesh(&mut self, _: &[Vertex], _: &[u32]) -> MeshHandle {
        let h = self.upload_calls;
        self.upload_calls += 1;
        h
    }
    fn draw_mesh(&mut self, _: MeshHandle, transform: Mat3, _: [f32; 4]) {
        self.draw_calls += 1;
        self.last_transform = Some(transform);
    }
    fn upload_texture(&mut self, _: &[u32], _: u32, _: u32) -> TextureHandle {
        self.upload_texture_calls += 1;
        self.upload_texture_calls as TextureHandle
    }
    fn free_texture(&mut self, _: TextureHandle) {
        self.free_texture_calls += 1;
    }
    fn draw_bitmap(&mut self, _: TextureHandle, transform: Mat3, _: [f32; 4]) {
        self.draw_bitmap_calls += 1;
        self.last_transform = Some(transform);
    }
    fn resize(&mut self, _: u32, _: u32) {}
    fn backend_name(&self) -> &'static str {
        "spy"
    }
    fn surface_size(&self) -> (u32, u32) {
        (800, 600)
    }
}

// ---------------------------------------------------------------------------
// Color
// ---------------------------------------------------------------------------

#[test]
fn color_rgba_components() {
    let c = Color::rgba(0.1, 0.2, 0.3, 0.4);
    assert!(approx(c.r, 0.1, 1e-6));
    assert!(approx(c.g, 0.2, 1e-6));
    assert!(approx(c.b, 0.3, 1e-6));
    assert!(approx(c.a, 0.4, 1e-6));
}

#[test]
fn color_hex_parses_rrggbbaa() {
    let c = Color::hex(0xFF8040A0);
    assert!(approx(c.r, 1.0, 2.0 / 255.0));
    assert!(approx(c.g, 0.502, 2.0 / 255.0));
    assert!(approx(c.b, 0.251, 2.0 / 255.0));
    assert!(approx(c.a, 0.627, 2.0 / 255.0));
}

#[test]
fn color_hex_fully_opaque_white() {
    assert_eq!(Color::hex(0xFFFFFFFF), Color::WHITE);
}

#[test]
fn color_hex_fully_transparent() {
    assert_eq!(Color::hex(0x00000000), Color::TRANSPARENT);
}

#[test]
fn color_with_alpha_preserves_rgb() {
    let c = Color::rgba(0.5, 0.6, 0.7, 1.0).with_alpha(0.0);
    assert!(approx(c.r, 0.5, 1e-6));
    assert!(approx(c.g, 0.6, 1e-6));
    assert!(approx(c.b, 0.7, 1e-6));
    assert!(approx(c.a, 0.0, 1e-6));
}

#[test]
fn color_to_array_round_trips() {
    let (r, g, b, a) = (0.1, 0.4, 0.7, 0.9);
    assert_eq!(Color::rgba(r, g, b, a).to_array(), [r, g, b, a]);
}

#[test]
fn color_black_constant() {
    let c = Color::BLACK;
    assert_eq!([c.r, c.g, c.b, c.a], [0.0, 0.0, 0.0, 1.0]);
}

#[test]
fn color_white_constant() {
    let c = Color::WHITE;
    assert_eq!([c.r, c.g, c.b, c.a], [1.0, 1.0, 1.0, 1.0]);
}

#[test]
fn color_transparent_constant() {
    assert_eq!(Color::TRANSPARENT.a, 0.0);
}

// ---------------------------------------------------------------------------
// Transform
// ---------------------------------------------------------------------------

#[test]
fn transform_identity_is_noop() {
    let p = Vec2::new(3.0, 7.0);
    assert_eq!(Transform::identity().apply(p), p);
}

#[test]
fn transform_translate() {
    let t = Transform::translate(3.0, 4.0);
    assert_eq!(t.apply(Vec2::ZERO), Vec2::new(3.0, 4.0));
}

#[test]
fn transform_translate_non_origin() {
    let t = Transform::translate(1.0, 2.0);
    assert_eq!(t.apply(Vec2::new(10.0, 20.0)), Vec2::new(11.0, 22.0));
}

#[test]
fn transform_rotate_quarter_turn() {
    let t = Transform::rotate(PI / 2.0);
    let r = t.apply(Vec2::new(1.0, 0.0));
    assert!(approx_vec(r, Vec2::new(0.0, 1.0), 1e-5));
}

#[test]
fn transform_rotate_full_turn() {
    let t = Transform::rotate(TAU);
    let r = t.apply(Vec2::new(1.0, 1.0));
    assert!(approx_vec(r, Vec2::new(1.0, 1.0), 1e-5));
}

#[test]
fn transform_scale() {
    let t = Transform::scale(2.0, 3.0);
    assert_eq!(t.apply(Vec2::new(1.0, 1.0)), Vec2::new(2.0, 3.0));
}

#[test]
fn transform_scale_non_uniform() {
    let t = Transform::scale(0.5, 4.0);
    assert_eq!(t.apply(Vec2::new(6.0, 2.0)), Vec2::new(3.0, 8.0));
}

#[test]
fn transform_compose_order() {
    let tx = Transform::translate(1.0, 0.0);
    let sc = Transform::scale(2.0, 2.0);
    let composed = tx.then(sc).apply(Vec2::ZERO);
    let manual = sc.apply(tx.apply(Vec2::ZERO));
    assert!(approx_vec(composed, manual, 1e-5));
    assert!(approx_vec(composed, Vec2::new(2.0, 0.0), 1e-5));
}

#[test]
fn transform_inverse_round_trip() {
    let t = Transform::translate(3.0, -2.0).then(Transform::rotate(0.7));
    let p = Vec2::new(5.0, 5.0);
    let roundtrip = t.then(t.inverse().unwrap()).apply(p);
    assert!(approx_vec(roundtrip, p, 1e-4));
}

#[test]
fn transform_singular_has_no_inverse() {
    assert!(Transform(Mat3::ZERO).inverse().is_none());
}

#[test]
fn transform_default_is_identity() {
    let p = Vec2::new(9.0, -3.0);
    assert_eq!(
        Transform::default().apply(p),
        Transform::identity().apply(p)
    );
}

// ---------------------------------------------------------------------------
// PathBuilder
// ---------------------------------------------------------------------------

fn seg_count(path: &Path) -> usize {
    path.segment_count()
}

#[test]
fn path_builder_empty_is_open() {
    assert!(!PathBuilder::new().build().is_closed());
}

#[test]
fn path_builder_empty_has_no_segments() {
    assert_eq!(seg_count(&PathBuilder::new().build()), 0);
}

#[test]
fn path_builder_close_is_closed() {
    let p = PathBuilder::new()
        .move_to(Vec2::ZERO)
        .line_to(Vec2::X)
        .close();
    assert!(p.is_closed());
}

#[test]
fn path_builder_build_is_open() {
    let p = PathBuilder::new()
        .move_to(Vec2::ZERO)
        .line_to(Vec2::X)
        .build();
    assert!(!p.is_closed());
}

#[test]
fn path_builder_move_to_does_not_close() {
    assert!(!PathBuilder::new().move_to(Vec2::ZERO).build().is_closed());
}

#[test]
fn path_builder_segment_count_move_line() {
    let p = PathBuilder::new()
        .move_to(Vec2::ZERO)
        .line_to(Vec2::X)
        .build();
    assert_eq!(seg_count(&p), 2);
}

#[test]
fn path_builder_quad_cubic_arc_counted() {
    let p = PathBuilder::new()
        .move_to(Vec2::ZERO)
        .quad_to(Vec2::new(0.5, 1.0), Vec2::X)
        .cubic_to(
            Vec2::new(0.2, 0.5),
            Vec2::new(0.8, 0.5),
            Vec2::new(2.0, 0.0),
        )
        .arc_to(Vec2::ZERO, 1.0, 0.0, PI)
        .build();
    assert_eq!(seg_count(&p), 4);
}

// ---------------------------------------------------------------------------
// Shape Primitives
// ---------------------------------------------------------------------------

#[test]
fn circle_is_closed() {
    assert!(circle(Vec2::ZERO, 1.0).is_closed());
}
#[test]
fn ellipse_is_closed() {
    assert!(ellipse(Vec2::ZERO, 2.0, 1.0).is_closed());
}
#[test]
fn rect_is_closed() {
    assert!(rect(Vec2::ZERO, Vec2::ONE).is_closed());
}
#[test]
fn rounded_rect_is_closed() {
    assert!(rounded_rect(Vec2::ZERO, Vec2::splat(4.0), 0.5).is_closed());
}
#[test]
fn regular_polygon_is_closed() {
    assert!(regular_polygon(Vec2::ZERO, 1.0, 5).is_closed());
}
#[test]
fn star_is_closed() {
    assert!(star(Vec2::ZERO, 2.0, 1.0, 5).is_closed());
}
#[test]
fn line_is_open() {
    assert!(!line(Vec2::ZERO, Vec2::X).is_closed());
}
#[test]
fn polyline_is_open() {
    assert!(!polyline(&[Vec2::ZERO, Vec2::X, Vec2::Y]).is_closed());
}
#[test]
fn polygon_is_closed() {
    assert!(polygon(&[Vec2::ZERO, Vec2::X, Vec2::Y]).is_closed());
}

#[test]
fn rounded_rect_zero_radius_is_closed() {
    // rounded_rect always emits cubic corners even when r=0 (collapsed to a point),
    // so segment count differs from rect — but the path must still be closed.
    let p = rounded_rect(Vec2::ZERO, Vec2::splat(4.0), 0.0);
    assert!(p.is_closed());
    assert!(seg_count(&p) > 0);
}

#[test]
fn regular_polygon_segment_count() {
    let n = 6u32;
    let p = regular_polygon(Vec2::ZERO, 1.0, n);
    // move_to + (n-1) line_to → n segments (close sets the flag, adds no segment)
    assert_eq!(seg_count(&p), n as usize);
}

#[test]
#[should_panic]
fn regular_polygon_requires_three_sides() {
    let _ = regular_polygon(Vec2::ZERO, 1.0, 2);
}

#[test]
#[should_panic]
fn star_requires_two_points() {
    let _ = star(Vec2::ZERO, 2.0, 1.0, 1);
}

#[test]
#[should_panic]
fn polygon_requires_three_points() {
    let _ = polygon(&[Vec2::ZERO, Vec2::X]);
}

// ---------------------------------------------------------------------------
// Style
// ---------------------------------------------------------------------------

#[test]
fn style_filled_has_fill_no_stroke() {
    let s = Style::filled(Color::WHITE);
    assert!(s.fill.is_some());
    assert!(s.stroke.is_none());
}

#[test]
fn style_stroked_has_stroke_no_fill() {
    let s = Style::stroked(Stroke::solid(Color::BLACK, 1.0));
    assert!(s.fill.is_none());
    assert!(s.stroke.is_some());
}

#[test]
fn stroke_solid_defaults() {
    let s = Stroke::solid(Color::BLACK, 2.0);
    assert_eq!(s.cap, LineCap::Butt);
    assert_eq!(s.join, LineJoin::Miter);
}

#[test]
fn style_no_fill_no_stroke_constructible() {
    let _ = Style {
        fill: None,
        stroke: None,
    };
}

// ---------------------------------------------------------------------------
// Scene Graph — structural (no rendering required)
// ---------------------------------------------------------------------------

#[test]
fn scene_new_is_empty() {
    assert!(Scene::new().root.children.is_empty());
}

#[test]
fn scene_add_increases_child_count() {
    let mut s = Scene::new();
    let shape = Shape::new(rect(Vec2::ZERO, Vec2::ONE), Style::filled(Color::WHITE));
    s.add(shape);
    assert_eq!(s.root.children.len(), 1);
}

#[test]
fn scene_add_group() {
    let mut s = Scene::new();
    s.add(Node::Group(Group::new()));
    assert_eq!(s.root.children.len(), 1);
}

#[test]
fn scene_render_empty_no_driver_calls() {
    let mut spy = SpyDriver::default();
    Scene::new().render(&mut spy);
    assert_eq!(spy.upload_calls, 0);
    assert_eq!(spy.draw_calls, 0);
}

#[test]
fn scene_render_calls_begin_and_end_frame() {
    let mut spy = SpyDriver::default();
    let mut s = Scene::new();
    s.add(Shape::new(
        rect(Vec2::ZERO, Vec2::ONE),
        Style::filled(Color::WHITE),
    ));
    s.render(&mut spy);
    assert_eq!(spy.begin_frames, 1);
    assert_eq!(spy.end_frames, 1);
}

#[test]
fn scene_render_filled_shape_calls_upload_and_draw() {
    let mut spy = SpyDriver::default();
    let mut s = Scene::new();
    s.add(Shape::new(
        rect(Vec2::ZERO, Vec2::ONE),
        Style::filled(Color::WHITE),
    ));
    s.render(&mut spy);
    assert!(spy.upload_calls >= 1);
    assert!(spy.draw_calls >= 1);
}

#[test]
fn scene_render_fill_and_stroke_calls_draw_twice() {
    let mut spy = SpyDriver::default();
    let mut s = Scene::new();
    let style = Style {
        fill: Some(gfx::style::Fill::Solid(Color::WHITE)),
        stroke: Some(Stroke::solid(Color::BLACK, 1.0)),
    };
    s.add(Shape::new(rect(Vec2::ZERO, Vec2::ONE), style));
    s.render(&mut spy);
    assert!(spy.draw_calls >= 2);
}

#[test]
fn scene_group_transform_composes() {
    let mut spy = SpyDriver::default();
    let mut s = Scene::new();
    let group = Group::new()
        .with_transform(Transform::translate(1.0, 0.0))
        .add(Node::Shape(
            Shape::new(rect(Vec2::ZERO, Vec2::ONE), Style::filled(Color::WHITE))
                .with_transform(Transform::scale(2.0, 2.0)),
        ));
    s.add(group);
    s.render(&mut spy);

    // The draw transform should encode both the group and shape transforms.
    let t = spy.last_transform.expect("no draw call");
    let expected = Transform::translate(1.0, 0.0)
        .then(Transform::scale(2.0, 2.0))
        .to_mat3();
    // Compare by applying both to a point.
    let p = Vec2::new(1.0, 1.0);
    let a = t.transform_point2(p);
    let b = expected.transform_point2(p);
    assert!(approx_vec(a, b, 1e-4));
}

// ---------------------------------------------------------------------------
// Parametric Path API
// All tests use a straight horizontal line of length 10: (0,0) → (10,0).
// These require path::parametric to be implemented.
// ---------------------------------------------------------------------------

fn hline() -> Path {
    PathBuilder::new()
        .move_to(Vec2::ZERO)
        .line_to(Vec2::new(10.0, 0.0))
        .build()
}

#[test]
fn path_length_nonzero_for_line() {
    assert!(approx(hline().length(), 10.0, 0.01));
}

#[test]
fn path_point_at_start() {
    assert!(approx_vec(hline().point_at(0.0), Vec2::ZERO, 1e-4));
}

#[test]
fn path_point_at_end() {
    assert!(approx_vec(
        hline().point_at(1.0),
        Vec2::new(10.0, 0.0),
        1e-4
    ));
}

#[test]
fn path_point_at_midpoint() {
    assert!(approx_vec(hline().point_at(0.5), Vec2::new(5.0, 0.0), 1e-4));
}

#[test]
fn path_tangent_at_is_unit_length() {
    let p = hline();
    for t in [0.0, 0.25, 0.5, 0.75, 1.0] {
        assert!(approx(p.tangent_at(t).length(), 1.0, 1e-4));
    }
}

#[test]
fn path_normal_perpendicular_to_tangent() {
    let p = hline();
    for t in [0.0, 0.25, 0.5, 0.75, 1.0] {
        let dot = p.tangent_at(t).dot(p.normal_at(t));
        assert!(approx(dot, 0.0, 1e-4));
    }
}

#[test]
fn path_split_at_half_lengths() {
    let p = hline();
    let (a, b) = p.split_at(0.5);
    assert!(approx(a.length(), 5.0, 0.1));
    assert!(approx(b.length(), 5.0, 0.1));
}

#[test]
fn path_trim_length() {
    let p = hline();
    let trimmed = p.trim(0.25, 0.75);
    assert!(approx(
        trimmed.length(),
        p.length() * 0.5,
        p.length() * 0.01
    ));
}

#[test]
fn path_reverse_start_is_original_end() {
    let p = hline();
    assert!(approx_vec(p.reverse().point_at(0.0), p.point_at(1.0), 1e-4));
}

#[test]
fn path_closed_point_at_one_returns_start() {
    let p = PathBuilder::new()
        .move_to(Vec2::ZERO)
        .line_to(Vec2::new(5.0, 0.0))
        .line_to(Vec2::new(5.0, 5.0))
        .close();
    assert!(approx_vec(p.point_at(1.0), p.point_at(0.0), 1e-4));
}

#[test]
fn path_offset_distance() {
    let p = hline();
    let offset = p.offset(5.0);
    // Sample at a few points and check each is ~5 units from the original line.
    for t in [0.1, 0.3, 0.5, 0.7, 0.9] {
        let op = offset.point_at(t);
        // The original line is y=0, x in [0,10], so distance from line = |op.y|.
        assert!(approx(op.y.abs(), 5.0, 0.5));
    }
}

#[test]
fn texture_handle_independent_of_mesh() {
    let mut spy = SpyDriver::default();
    spy.begin_frame();
    let mh = spy.upload_mesh(&[], &[]);
    let th = spy.upload_texture(&[0xFF_FF_00_00], 1, 1);
    assert_eq!(mh, 0);
    assert_eq!(th, 1);
}
