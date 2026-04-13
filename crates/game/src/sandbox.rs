use gfx::{
    Color, Vec2,
    shape::{circle, line},
    style::{Fill, LineJoin, LineCap, Stroke, Style},
    tessellate,
};
use glam::Mat3;

// ---------------------------------------------------------------------------
// World-space configuration
// ---------------------------------------------------------------------------

/// World units visible from the centre to each viewport edge.
/// Changing this zooms the camera in or out.
const HALF_VIEW: f32 = 5.0;

/// Grid cell size in world units.
const TILE_SIZE: f32 = 0.1;

/// Player radius in world units.
const PLAYER_RADIUS: f32 = 0.5;

// ---------------------------------------------------------------------------
// World → NDC conversion
// ---------------------------------------------------------------------------

fn w(v: f32) -> f32 { v / HALF_VIEW }
fn wv(v: Vec2) -> Vec2 { Vec2::new(w(v.x), w(v.y)) }

// ---------------------------------------------------------------------------
// Scene entry point
// ---------------------------------------------------------------------------

/// Sandbox scene for game development — top-down camera, tiled floor.
pub fn draw_scene(driver: &mut dyn gfx::GraphicsDriver, fps: f32, player_pos: Vec2) {
    let backend = driver.backend_name(); // &'static str — no borrow held past this line
    driver.clear([0.13, 0.14, 0.12, 1.0]); // ground colour fills the viewport
    draw_grid(driver);
    draw_player(driver, player_pos);
    crate::hud::draw_fps(driver, fps);
    crate::hud::draw_backend(driver, backend);
}

// ---------------------------------------------------------------------------
// Floor grid
// ---------------------------------------------------------------------------

fn draw_grid(driver: &mut dyn gfx::GraphicsDriver) {
    let grid_style = Style::stroked(Stroke::solid(Color::hex(0x000000FF), 0.003));

    // Integer tile indices that span the visible area.
    let first = (-HALF_VIEW / TILE_SIZE).floor() as i32;
    let last  = ( HALF_VIEW / TILE_SIZE).ceil()  as i32;

    for i in first..=last {
        let coord = w(i as f32 * TILE_SIZE);

        // Vertical line at world-x = i * TILE_SIZE.
        draw_shape(
            driver,
            &line(Vec2::new(coord, -1.0), Vec2::new(coord, 1.0)),
            &grid_style,
            Mat3::IDENTITY,
        );

        // Horizontal line at world-y = i * TILE_SIZE.
        draw_shape(
            driver,
            &line(Vec2::new(-1.0, coord), Vec2::new(1.0, coord)),
            &grid_style,
            Mat3::IDENTITY,
        );
    }
}

// ---------------------------------------------------------------------------
// Player
// ---------------------------------------------------------------------------

fn draw_player(driver: &mut dyn gfx::GraphicsDriver, pos: Vec2) {
    draw_shape(
        driver,
        &circle(wv(pos), w(PLAYER_RADIUS)),
        &Style {
            fill:   Some(Fill::Solid(Color::hex(0x2979FFFF))),
            stroke: Some(Stroke {
                color: Color::hex(0x000000FF),
                width: 0.008,
                cap:   LineCap::Round,
                join:  LineJoin::Round,
            }),
        },
        Mat3::IDENTITY,
    );
}

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn draw_shape(
    driver:    &mut dyn gfx::GraphicsDriver,
    path:      &gfx::Path,
    style:     &gfx::Style,
    transform: Mat3,
) {
    for mesh in tessellate(path, style) {
        let handle = driver.upload_mesh(&mesh.vertices, &mesh.indices);
        driver.draw_mesh(handle, transform, [1.0, 1.0, 1.0, 1.0]);
    }
}
