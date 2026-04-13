# gfx Crate Tests

Every spec claim in `index.md` has at least one corresponding test below.
Tests are grouped by the spec section they cover.

---

## Color

### `color_rgba_components`
`Color::rgba(0.1, 0.2, 0.3, 0.4)` stores r=0.1, g=0.2, b=0.3, a=0.4 exactly.

### `color_hex_parses_rrggbbaa`
`Color::hex(0xFF8040A0)` → r≈1.0, g≈0.502, b≈0.251, a≈0.627 (each within 1/255).

### `color_hex_fully_opaque_white`
`Color::hex(0xFFFFFFFF)` == `Color::WHITE`.

### `color_hex_fully_transparent`
`Color::hex(0x00000000)` == `Color::TRANSPARENT`.

### `color_with_alpha_preserves_rgb`
`Color::rgba(0.5, 0.6, 0.7, 1.0).with_alpha(0.0)` has r=0.5, g=0.6, b=0.7, a=0.0.

### `color_to_array_round_trips`
`Color::rgba(r, g, b, a).to_array()` == `[r, g, b, a]`.

### `color_black_constant`
`Color::BLACK` has r=0, g=0, b=0, a=1.

### `color_white_constant`
`Color::WHITE` has r=1, g=1, b=1, a=1.

### `color_transparent_constant`
`Color::TRANSPARENT` has a=0.

---

## Transform

### `transform_identity_is_noop`
`Transform::identity().apply(Vec2::new(3.0, 7.0))` == `Vec2::new(3.0, 7.0)`.

### `transform_translate`
`Transform::translate(3.0, 4.0).apply(Vec2::ZERO)` == `Vec2::new(3.0, 4.0)`.

### `transform_translate_non_origin`
`Transform::translate(1.0, 2.0).apply(Vec2::new(10.0, 20.0))` == `Vec2::new(11.0, 22.0)`.

### `transform_rotate_quarter_turn`
`Transform::rotate(PI/2).apply(Vec2::new(1.0, 0.0))` ≈ `Vec2::new(0.0, 1.0)` (within 1e-5).

### `transform_rotate_full_turn`
`Transform::rotate(2*PI).apply(Vec2::new(1.0, 1.0))` ≈ `Vec2::new(1.0, 1.0)` (within 1e-5).

### `transform_scale`
`Transform::scale(2.0, 3.0).apply(Vec2::new(1.0, 1.0))` == `Vec2::new(2.0, 3.0)`.

### `transform_scale_non_uniform`
`Transform::scale(0.5, 4.0).apply(Vec2::new(6.0, 2.0))` == `Vec2::new(3.0, 8.0)`.

### `transform_compose_order`
`Transform::translate(1.0, 0.0).then(Transform::scale(2.0, 2.0)).apply(Vec2::ZERO)`
== `Transform::scale(2.0, 2.0).apply(Transform::translate(1.0, 0.0).apply(Vec2::ZERO))`
== `Vec2::new(2.0, 0.0)`.

### `transform_inverse_round_trip`
For `t = Transform::translate(3.0, -2.0).then(Transform::rotate(0.7))`:
`t.then(t.inverse().unwrap()).apply(Vec2::new(5.0, 5.0))` ≈ `Vec2::new(5.0, 5.0)` (within 1e-4).

### `transform_singular_has_no_inverse`
`Transform(Mat3::ZERO).inverse()` == `None`.

### `transform_default_is_identity`
`Transform::default()` == `Transform::identity()`.

---

## PathBuilder

### `path_builder_empty_is_open`
`PathBuilder::new().build().is_closed()` == `false`.

### `path_builder_empty_has_no_segments`
`PathBuilder::new().build()` produces a path with zero segments.

### `path_builder_close_is_closed`
`PathBuilder::new().move_to(p).line_to(q).close().is_closed()` == `true`.

### `path_builder_build_is_open`
`PathBuilder::new().move_to(p).line_to(q).build().is_closed()` == `false`.

### `path_builder_move_to_does_not_close`
A path with only `move_to` followed by `build()` is open.

### `path_builder_segment_count_move_line`
`PathBuilder::new().move_to(p).line_to(q).build()` has 2 segments.

### `path_builder_quad_cubic_arc_counted`
A path built with one of each (`move_to`, `quad_to`, `cubic_to`, `arc_to`) has 4 segments.

---

## Shape Primitives

### `circle_is_closed`
`circle(center, r).is_closed()` == `true`.

### `ellipse_is_closed`
`ellipse(center, rx, ry).is_closed()` == `true`.

### `rect_is_closed`
`rect(origin, size).is_closed()` == `true`.

### `rounded_rect_is_closed`
`rounded_rect(origin, size, r).is_closed()` == `true`.

### `rounded_rect_zero_radius_equals_rect_segment_count`
`rounded_rect(origin, size, 0.0)` and `rect(origin, size)` produce the same number of segments
(the corners collapse to lines).

### `regular_polygon_is_closed`
`regular_polygon(center, r, 5).is_closed()` == `true`.

### `regular_polygon_requires_three_sides`
`regular_polygon(center, r, 2)` panics.

### `regular_polygon_segment_count`
`regular_polygon(center, r, n)` has `n + 1` segments (n line_to + 1 move_to).

### `star_is_closed`
`star(center, outer, inner, 5).is_closed()` == `true`.

### `star_requires_two_points`
`star(center, outer, inner, 1)` panics.

### `line_is_open`
`line(start, end).is_closed()` == `false`.

### `polyline_is_open`
`polyline(&[a, b, c]).is_closed()` == `false`.

### `polygon_is_closed`
`polygon(&[a, b, c]).is_closed()` == `true`.

### `polygon_requires_three_points`
`polygon(&[a, b])` panics.

---

## Style

### `style_filled_has_fill_no_stroke`
`Style::filled(Color::RED)` has `fill = Some(_)` and `stroke = None`.

### `style_stroked_has_stroke_no_fill`
`Style::stroked(Stroke::solid(Color::BLACK, 1.0))` has `stroke = Some(_)` and `fill = None`.

### `stroke_solid_defaults`
`Stroke::solid(color, w)` has `cap = LineCap::Butt` and `join = LineJoin::Miter`.

### `style_no_fill_no_stroke`
A `Style { fill: None, stroke: None }` is constructible (no panic).

---

## Scene Graph

### `scene_new_is_empty`
`Scene::new().root.children` is empty.

### `scene_add_increases_child_count`
After `scene.add(shape)`, `scene.root.children.len()` == 1.

### `scene_render_empty_no_driver_calls`
`Scene::new().render(&mut spy_driver)` makes zero calls to `begin_frame`, `upload_mesh`,
`draw_mesh`, or `end_frame`.

### `scene_render_calls_begin_and_end_frame`
A scene with one filled shape calls `begin_frame` once and `end_frame` once.
**Requires tessellation.**

### `scene_render_filled_shape_calls_upload_and_draw`
A scene with one filled shape calls `upload_mesh` and `draw_mesh` at least once.
**Requires tessellation.**

### `scene_render_both_fill_and_stroke_calls_draw_twice`
A scene with a shape that has both fill and stroke calls `draw_mesh` at least twice.
**Requires tessellation.**

### `scene_group_transform_composes`
A shape inside a group with transform `T_group`, with shape local transform `T_shape`,
is drawn with the combined transform `T_group ∘ T_shape`.
**Requires tessellation.**

---

## Parametric Path API

All parametric tests use a single `line_to` segment from `(0,0)` to `(10,0)` (a
horizontal line of known length = 10).

### `path_length_nonzero_for_line`
`line(Vec2::ZERO, Vec2::new(10.0, 0.0)).length()` ≈ 10.0 (within 0.01).

### `path_point_at_start`
`path.point_at(0.0)` == start point.

### `path_point_at_end`
`path.point_at(1.0)` == end point.

### `path_point_at_midpoint`
For a straight line from (0,0) to (10,0): `path.point_at(0.5)` ≈ `(5.0, 0.0)`.

### `path_tangent_at_is_unit_length`
`path.tangent_at(t).length()` ≈ 1.0 for any `t` in [0, 1].

### `path_normal_perpendicular_to_tangent`
`path.tangent_at(t).dot(path.normal_at(t))` ≈ 0.0.

### `path_split_at_half_lengths`
For a line of length 10: `split_at(0.5)` produces two paths each with length ≈ 5.0.

### `path_trim_length`
`path.trim(0.25, 0.75).length()` ≈ `path.length() * 0.5` (within 1%).

### `path_reverse_start_is_original_end`
`path.reverse().point_at(0.0)` == `path.point_at(1.0)`.

### `path_closed_point_at_one_returns_start`
A closed path: `path.point_at(1.0)` == `path.point_at(0.0)`.

### `path_offset_distance`
For a horizontal line, `path.offset(5.0)` — every sampled point on the offset path
is within 0.1 of 5.0 units from the nearest point on the original path.
