use crate::{Collider, Contact};
use glam::Vec2;

/// Dispatch narrowphase detection between two positioned colliders.
///
/// Returns `Some(Contact)` where `normal` points from A toward B,
/// or `None` when the shapes are separated.
pub fn detect(pos_a: Vec2, col_a: &Collider, pos_b: Vec2, col_b: &Collider) -> Option<Contact> {
    use Collider::*;
    match (col_a, col_b) {
        (Circle { radius: ra }, Circle { radius: rb }) => circle_circle(pos_a, *ra, pos_b, *rb),

        // circle_convex returns normal pointing from polygon toward circle.
        // For (Circle=A, Convex=B) that is B→A — flip to get A→B.
        // For (Convex=A, Circle=B) that is A→B — already correct, no flip.
        (Circle { radius }, Convex { vertices }) => {
            circle_convex(pos_a, *radius, pos_b, vertices).map(flip)
        }
        (Convex { vertices }, Circle { radius }) => circle_convex(pos_b, *radius, pos_a, vertices),

        // Same logic applies to circle_mesh.
        (Circle { radius }, Mesh { vertices, indices }) => {
            circle_mesh(pos_a, *radius, pos_b, vertices, indices).map(flip)
        }
        (Mesh { vertices, indices }, Circle { radius }) => {
            circle_mesh(pos_b, *radius, pos_a, vertices, indices)
        }

        (Convex { vertices: va }, Convex { vertices: vb }) => convex_convex(pos_a, va, pos_b, vb),

        (
            Convex { vertices },
            Mesh {
                vertices: mv,
                indices,
            },
        ) => convex_mesh(pos_a, vertices, pos_b, mv, indices),
        (
            Mesh {
                vertices: mv,
                indices,
            },
            Convex { vertices },
        ) => convex_mesh(pos_b, vertices, pos_a, mv, indices).map(flip),

        // Mesh–Mesh is not supported.
        (Mesh { .. }, Mesh { .. }) => None,
    }
}

// ---------------------------------------------------------------------------
// Primitive tests
// ---------------------------------------------------------------------------

fn circle_circle(pos_a: Vec2, r_a: f32, pos_b: Vec2, r_b: f32) -> Option<Contact> {
    let diff = pos_b - pos_a;
    let dist_sq = diff.length_squared();
    let combined = r_a + r_b;
    if dist_sq >= combined * combined {
        return None;
    }
    let dist = dist_sq.sqrt();
    let normal = if dist > 1e-7 { diff / dist } else { Vec2::Y };
    Some(Contact {
        normal,
        depth: combined - dist,
        point: pos_a + normal * r_a,
    })
}

/// SAT test between a circle and a convex polygon.
///
/// `local_verts` are in local (body) space; `poly_pos` is the body's world position.
/// The returned normal points from the polygon toward the circle.
fn circle_convex(
    circle_pos: Vec2,
    radius: f32,
    poly_pos: Vec2,
    local_verts: &[Vec2],
) -> Option<Contact> {
    let n = local_verts.len();
    if n < 2 {
        return None;
    }

    // Transform polygon vertices to world space once.
    let verts: Vec<Vec2> = local_verts.iter().map(|&v| poly_pos + v).collect();

    let mut min_depth = f32::MAX;
    let mut best_axis = Vec2::ZERO;

    // --- Edge-normal axes ---
    for i in 0..n {
        let edge = verts[(i + 1) % n] - verts[i];
        let axis = match Vec2::new(-edge.y, edge.x).try_normalize() {
            Some(a) => a,
            None => continue, // degenerate edge
        };

        match overlap_circle_poly(circle_pos, radius, &verts, axis) {
            None => return None, // separating axis found
            Some(depth) if depth < min_depth => {
                min_depth = depth;
                best_axis = axis;
            }
            _ => {}
        }
    }

    // --- Nearest-vertex axis (handles vertex-region collisions) ---
    if let Some(&nearest) = verts.iter().min_by(|a, b| {
        a.distance_squared(circle_pos)
            .partial_cmp(&b.distance_squared(circle_pos))
            .unwrap()
    }) {
        let to_center = circle_pos - nearest;
        if let Some(axis) = to_center.try_normalize() {
            match overlap_circle_poly(circle_pos, radius, &verts, axis) {
                None => return None,
                Some(depth) if depth < min_depth => {
                    min_depth = depth;
                    best_axis = axis;
                }
                _ => {}
            }
        }
    }

    // Orient the normal from the polygon centroid toward the circle.
    let centroid: Vec2 = verts.iter().copied().sum::<Vec2>() / n as f32;
    if best_axis.dot(circle_pos - centroid) < 0.0 {
        best_axis = -best_axis;
    }

    Some(Contact {
        normal: best_axis,
        depth: min_depth,
        point: circle_pos - best_axis * radius,
    })
}

/// SAT between two convex polygons, both in local space with their world positions.
fn convex_convex(pos_a: Vec2, local_a: &[Vec2], pos_b: Vec2, local_b: &[Vec2]) -> Option<Contact> {
    let wa: Vec<Vec2> = local_a.iter().map(|&v| pos_a + v).collect();
    let wb: Vec<Vec2> = local_b.iter().map(|&v| pos_b + v).collect();

    let mut min_depth = f32::MAX;
    let mut best_axis = Vec2::ZERO;

    // Axes from A's edges.
    for i in 0..wa.len() {
        let edge = wa[(i + 1) % wa.len()] - wa[i];
        let axis = match Vec2::new(-edge.y, edge.x).try_normalize() {
            Some(a) => a,
            None => continue,
        };
        match overlap_poly_poly(&wa, &wb, axis) {
            None => return None,
            Some(depth) if depth < min_depth => {
                min_depth = depth;
                best_axis = axis;
            }
            _ => {}
        }
    }

    // Axes from B's edges.
    for i in 0..wb.len() {
        let edge = wb[(i + 1) % wb.len()] - wb[i];
        let axis = match Vec2::new(-edge.y, edge.x).try_normalize() {
            Some(a) => a,
            None => continue,
        };
        match overlap_poly_poly(&wa, &wb, axis) {
            None => return None,
            Some(depth) if depth < min_depth => {
                min_depth = depth;
                best_axis = axis;
            }
            _ => {}
        }
    }

    // Orient normal from A centroid toward B centroid.
    let ca: Vec2 = wa.iter().copied().sum::<Vec2>() / wa.len() as f32;
    let cb: Vec2 = wb.iter().copied().sum::<Vec2>() / wb.len() as f32;
    if best_axis.dot(cb - ca) < 0.0 {
        best_axis = -best_axis;
    }

    // Contact point: deepest vertex of B projected onto the contact normal.
    let point = *wb
        .iter()
        .min_by(|a, b| a.dot(best_axis).partial_cmp(&b.dot(best_axis)).unwrap())
        .unwrap_or(&Vec2::ZERO);

    Some(Contact {
        normal: best_axis,
        depth: min_depth,
        point,
    })
}

/// Circle vs triangle mesh — test each triangle; return shallowest contact.
fn circle_mesh(
    circle_pos: Vec2,
    radius: f32,
    mesh_pos: Vec2,
    vertices: &[Vec2],
    indices: &[[u32; 3]],
) -> Option<Contact> {
    indices
        .iter()
        .filter_map(|tri| {
            let tri_verts = [
                vertices[tri[0] as usize],
                vertices[tri[1] as usize],
                vertices[tri[2] as usize],
            ];
            circle_convex(circle_pos, radius, mesh_pos, &tri_verts)
        })
        .min_by(|a, b| a.depth.partial_cmp(&b.depth).unwrap())
}

/// Convex polygon vs triangle mesh — test each triangle; return shallowest contact.
fn convex_mesh(
    poly_pos: Vec2,
    poly_verts: &[Vec2],
    mesh_pos: Vec2,
    mesh_verts: &[Vec2],
    indices: &[[u32; 3]],
) -> Option<Contact> {
    indices
        .iter()
        .filter_map(|tri| {
            let tri_verts = [
                mesh_verts[tri[0] as usize],
                mesh_verts[tri[1] as usize],
                mesh_verts[tri[2] as usize],
            ];
            convex_convex(poly_pos, poly_verts, mesh_pos, &tri_verts)
        })
        .min_by(|a, b| a.depth.partial_cmp(&b.depth).unwrap())
}

// ---------------------------------------------------------------------------
// SAT helpers
// ---------------------------------------------------------------------------

/// Project a polygon onto `axis`; returns (min, max).
fn project_poly(verts: &[Vec2], axis: Vec2) -> (f32, f32) {
    let mut min = f32::MAX;
    let mut max = f32::NEG_INFINITY;
    for v in verts {
        let p = v.dot(axis);
        if p < min {
            min = p;
        }
        if p > max {
            max = p;
        }
    }
    (min, max)
}

/// Returns overlap depth if the circle and polygon projections overlap on `axis`,
/// or `None` if they are separated.
fn overlap_circle_poly(circle_pos: Vec2, radius: f32, poly: &[Vec2], axis: Vec2) -> Option<f32> {
    let (poly_min, poly_max) = project_poly(poly, axis);
    let c = circle_pos.dot(axis);
    let cmin = c - radius;
    let cmax = c + radius;
    if cmax <= poly_min || poly_max <= cmin {
        return None;
    }
    Some((cmax - poly_min).min(poly_max - cmin))
}

/// Returns overlap depth if both polygon projections overlap on `axis`, or `None`.
fn overlap_poly_poly(a: &[Vec2], b: &[Vec2], axis: Vec2) -> Option<f32> {
    let (min_a, max_a) = project_poly(a, axis);
    let (min_b, max_b) = project_poly(b, axis);
    if max_a <= min_b || max_b <= min_a {
        return None;
    }
    Some((max_a - min_b).min(max_b - min_a))
}

/// Flip a contact so the normal points the other way (used for symmetric dispatch).
fn flip(c: Contact) -> Contact {
    Contact {
        normal: -c.normal,
        ..c
    }
}
