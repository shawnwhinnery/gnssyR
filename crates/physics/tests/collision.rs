use glam::Vec2;
use physics::{narrow, Body, BodyHandle, Collider, PhysicsWorld};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn approx_eq(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-4
}

fn vec_approx_eq(a: Vec2, b: Vec2) -> bool {
    (a - b).length() < 1e-4
}

/// Unit square as a convex polygon (CCW, centred at origin).
fn unit_square() -> Vec<Vec2> {
    vec![
        Vec2::new(-0.5, -0.5),
        Vec2::new(0.5, -0.5),
        Vec2::new(0.5, 0.5),
        Vec2::new(-0.5, 0.5),
    ]
}

// ---------------------------------------------------------------------------
// Circle – Circle
// ---------------------------------------------------------------------------

#[test]
fn circle_circle_no_overlap() {
    let a = Collider::Circle { radius: 1.0 };
    let b = Collider::Circle { radius: 1.0 };
    // 3 units apart, combined radius 2 → separated
    let result = narrow::detect(Vec2::ZERO, &a, Vec2::new(3.0, 0.0), &b);
    assert!(result.is_none(), "expected no contact; got {result:?}");
}

#[test]
fn circle_circle_overlap() {
    let a = Collider::Circle { radius: 1.0 };
    let b = Collider::Circle { radius: 1.0 };
    // 1.5 units apart, combined radius 2 → 0.5 penetration
    let c = narrow::detect(Vec2::ZERO, &a, Vec2::new(1.5, 0.0), &b).expect("expected contact");
    assert!(
        approx_eq(c.depth, 0.5),
        "depth should be 0.5, got {}",
        c.depth
    );
    assert!(
        vec_approx_eq(c.normal, Vec2::X),
        "normal should be +X, got {:?}",
        c.normal
    );
}

#[test]
fn circle_circle_coincident() {
    // Circles at the same position — fallback normal is Vec2::Y.
    let col = Collider::Circle { radius: 1.0 };
    let c = narrow::detect(Vec2::ZERO, &col, Vec2::ZERO, &col)
        .expect("expected contact for coincident circles");
    assert!(
        vec_approx_eq(c.normal, Vec2::Y),
        "fallback normal should be Y, got {:?}",
        c.normal
    );
}

// ---------------------------------------------------------------------------
// Circle – Convex
// ---------------------------------------------------------------------------

#[test]
fn circle_convex_miss() {
    let circle = Collider::Circle { radius: 0.4 };
    let square = Collider::Convex {
        vertices: unit_square(),
    };
    // Circle is 2 units to the right of the square — no overlap
    let result = narrow::detect(Vec2::new(2.0, 0.0), &circle, Vec2::ZERO, &square);
    assert!(result.is_none(), "expected no contact; got {result:?}");
}

#[test]
fn circle_convex_hit_edge() {
    let circle = Collider::Circle { radius: 0.4 };
    let square = Collider::Convex {
        vertices: unit_square(),
    };
    // Circle centred at (0.7, 0) just overlaps the right edge of the unit square (edge at x=0.5).
    // Penetration depth = 0.5 + 0.4 - 0.7 = 0.2
    let c = narrow::detect(Vec2::new(0.7, 0.0), &circle, Vec2::ZERO, &square)
        .expect("expected contact on right edge");
    assert!(
        approx_eq(c.depth, 0.2),
        "depth should be ~0.2, got {}",
        c.depth
    );
    // Normal points from A (circle) toward B (square) = -X (circle is east of square)
    assert!(
        c.normal.dot(-Vec2::X) > 0.9,
        "normal should point roughly -X (A→B), got {:?}",
        c.normal
    );
}

#[test]
fn circle_convex_vertex_hit() {
    let circle = Collider::Circle { radius: 0.3 };
    let square = Collider::Convex {
        vertices: unit_square(),
    };
    // Circle positioned near the top-right corner (0.5, 0.5).
    // Place it at (0.7, 0.7) — distance to corner = 0.2√2 ≈ 0.283, radius = 0.3 → collision
    let c = narrow::detect(Vec2::new(0.7, 0.7), &circle, Vec2::ZERO, &square)
        .expect("expected contact near vertex");
    assert!(c.depth > 0.0, "depth should be positive, got {}", c.depth);
    // Normal points from A (circle, northeast) toward B (square, origin) = roughly (-1,-1)
    assert!(
        c.normal.dot(Vec2::new(-1.0, -1.0).normalize()) > 0.7,
        "normal should point roughly toward square (A→B), got {:?}",
        c.normal
    );
}

// ---------------------------------------------------------------------------
// Convex – Convex
// ---------------------------------------------------------------------------

#[test]
fn convex_convex_miss() {
    let sq = Collider::Convex {
        vertices: unit_square(),
    };
    // Squares 2 units apart on X — separated
    let result = narrow::detect(Vec2::ZERO, &sq, Vec2::new(2.0, 0.0), &sq);
    assert!(result.is_none(), "expected no contact; got {result:?}");
}

#[test]
fn convex_convex_hit() {
    let sq = Collider::Convex {
        vertices: unit_square(),
    };
    // Squares 0.8 units apart (each half-width 0.5, so 0.2 overlap)
    let c = narrow::detect(Vec2::ZERO, &sq, Vec2::new(0.8, 0.0), &sq).expect("expected contact");
    assert!(
        approx_eq(c.depth, 0.2),
        "depth should be ~0.2, got {}",
        c.depth
    );
    assert!(
        c.normal.dot(Vec2::X) > 0.9,
        "normal should point roughly +X, got {:?}",
        c.normal
    );
}

// ---------------------------------------------------------------------------
// Circle – Mesh
// ---------------------------------------------------------------------------

fn floor_mesh() -> Collider {
    // A single triangle representing a flat floor at y = 0.
    //   (-2, 0) → (2, 0) → (0, -1)
    Collider::Mesh {
        vertices: vec![
            Vec2::new(-2.0, 0.0),
            Vec2::new(2.0, 0.0),
            Vec2::new(0.0, -1.0),
        ],
        indices: vec![[0, 1, 2]],
    }
}

#[test]
fn circle_mesh_miss() {
    let circle = Collider::Circle { radius: 0.3 };
    let mesh = floor_mesh();
    // Circle at (0, 1) — well above the floor triangle
    let result = narrow::detect(Vec2::new(0.0, 1.0), &circle, Vec2::ZERO, &mesh);
    assert!(result.is_none(), "expected no contact; got {result:?}");
}

#[test]
fn circle_mesh_hit() {
    let circle = Collider::Circle { radius: 0.4 };
    let mesh = floor_mesh();
    // Circle centred at (0, 0.2) — top edge of triangle is at y=0, so 0.2 gap.
    // Circle radius 0.4 extends to y=-0.2, meaning it penetrates the triangle top edge.
    // Penetration ≈ 0.4 - 0.2 = 0.2
    let c = narrow::detect(Vec2::new(0.0, 0.2), &circle, Vec2::ZERO, &mesh)
        .expect("expected contact with floor mesh");
    assert!(c.depth > 0.0, "depth should be positive, got {}", c.depth);
}

// ---------------------------------------------------------------------------
// PhysicsWorld integration
// ---------------------------------------------------------------------------

#[test]
fn world_bounce() {
    let mut world = PhysicsWorld::new();

    // Two circles moving toward each other along X.
    // Centers 0.8 apart, combined radii 1.2 → 0.4 penetration.
    let a = world.add_body(Body {
        position: Vec2::new(-0.4, 0.0),
        velocity: Vec2::new(2.0, 0.0),
        mass: 1.0,
        restitution: 1.0, // elastic
        collider: Collider::Circle { radius: 0.6 },
    });
    let b = world.add_body(Body {
        position: Vec2::new(0.4, 0.0),
        velocity: Vec2::new(-2.0, 0.0),
        mass: 1.0,
        restitution: 1.0,
        collider: Collider::Circle { radius: 0.6 },
    });

    // dt=0 so positions stay; impulse resolves the overlap.
    world.step(0.0);

    let va = world.body(a).velocity;
    let vb = world.body(b).velocity;
    // After elastic collision of equal masses, velocities should swap.
    assert!(va.x < 0.0, "body A should bounce left, got vx={}", va.x);
    assert!(vb.x > 0.0, "body B should bounce right, got vx={}", vb.x);
    assert!(
        !world.contacts().is_empty(),
        "there should be at least one contact"
    );
}

#[test]
fn world_static_body_does_not_move() {
    let mut world = PhysicsWorld::new();

    // Static floor mesh
    let floor = world.add_body(Body {
        position: Vec2::ZERO,
        velocity: Vec2::ZERO,
        mass: f32::INFINITY,
        restitution: 0.5,
        collider: floor_mesh(),
    });

    // Dynamic circle falling toward the floor
    let ball = world.add_body(Body {
        position: Vec2::new(0.0, 0.3),
        velocity: Vec2::new(0.0, -1.0),
        mass: 1.0,
        restitution: 0.5,
        collider: Collider::Circle { radius: 0.4 },
    });

    world.step(0.016);

    let floor_pos = world.body(floor).position;
    assert!(
        vec_approx_eq(floor_pos, Vec2::ZERO),
        "static body should not move, pos={floor_pos:?}"
    );

    let ball_vel = world.body(ball).velocity;
    assert!(
        ball_vel.y > -1.0,
        "ball velocity should have been corrected upward, vy={}",
        ball_vel.y
    );
}

#[test]
fn world_contacts_cleared_between_steps() {
    let mut world = PhysicsWorld::new();

    // Two circles that overlap
    world.add_body(Body {
        position: Vec2::ZERO,
        velocity: Vec2::ZERO,
        mass: 1.0,
        restitution: 0.0,
        collider: Collider::Circle { radius: 1.0 },
    });
    world.add_body(Body {
        position: Vec2::new(0.5, 0.0),
        velocity: Vec2::ZERO,
        mass: 1.0,
        restitution: 0.0,
        collider: Collider::Circle { radius: 1.0 },
    });

    world.step(0.0);
    assert!(!world.contacts().is_empty());

    // Move bodies apart so they no longer collide
    world.body_mut(BodyHandle(0)).position = Vec2::new(-5.0, 0.0);
    world.body_mut(BodyHandle(1)).position = Vec2::new(5.0, 0.0);
    world.step(0.0);
    assert!(
        world.contacts().is_empty(),
        "contacts should be empty after separation"
    );
}
