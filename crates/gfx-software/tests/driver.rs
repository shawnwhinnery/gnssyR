use gfx::driver::{GraphicsDriver, Vertex};
use gfx_software::SoftwareDriver;
use glam::{Mat3, Vec2};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Convert a normalised [0,1]^4 colour to packed ARGB u32 the same way the
/// driver does: `(a<<24)|(r<<16)|(g<<8)|b`.
fn pack(r: f32, g: f32, b: f32, a: f32) -> u32 {
    let r = (r * 255.0) as u32;
    let g = (g * 255.0) as u32;
    let b = (b * 255.0) as u32;
    let a = (a * 255.0) as u32;
    (a << 24) | (r << 16) | (g << 8) | b
}

const RED:   u32 = 0xFF_FF_00_00;
const GREEN: u32 = 0xFF_00_FF_00;
const BLUE:  u32 = 0xFF_00_00_FF;
const WHITE: u32 = 0xFF_FF_FF_FF;
const BLACK: u32 = 0xFF_00_00_00;

/// Map a clip-space point to a pixel index in a W×H framebuffer.
fn clip_to_pixel(cx: f32, cy: f32, w: u32, h: u32) -> usize {
    let px = ((cx + 1.0) / 2.0 * w as f32).round() as usize;
    let py = ((1.0 - cy) / 2.0 * h as f32).round() as usize;
    let px = px.min(w as usize - 1);
    let py = py.min(h as usize - 1);
    py * w as usize + px
}

/// A simple 3-vertex triangle mesh — white fill, centred in clip space.
fn triangle_mesh() -> (Vec<Vertex>, Vec<u32>) {
    let verts = vec![
        Vertex { position: [ 0.0,  0.5], color: [1.0, 1.0, 1.0, 1.0] },
        Vertex { position: [-0.5, -0.5], color: [1.0, 1.0, 1.0, 1.0] },
        Vertex { position: [ 0.5, -0.5], color: [1.0, 1.0, 1.0, 1.0] },
    ];
    (verts, vec![0, 1, 2])
}

fn run_frame(driver: &mut SoftwareDriver, f: impl FnOnce(&mut SoftwareDriver)) {
    driver.begin_frame();
    f(driver);
    driver.end_frame();
    driver.present();
}

// ---------------------------------------------------------------------------
// Construction
// ---------------------------------------------------------------------------

#[test]
fn headless_construction() {
    let _ = SoftwareDriver::headless(200, 100);
}

#[test]
fn surface_size_matches_construction() {
    let d = SoftwareDriver::headless(200, 100);
    assert_eq!(d.surface_size(), (200, 100));
}

#[test]
fn initial_pixel_count() {
    let d = SoftwareDriver::headless(200, 100);
    assert_eq!(d.pixels().len(), 200 * 100);
}

// ---------------------------------------------------------------------------
// Frame Lifecycle
// ---------------------------------------------------------------------------

#[test]
fn frame_lifecycle_no_panic() {
    let mut d = SoftwareDriver::headless(64, 64);
    d.begin_frame();
    d.end_frame();
    d.present();
}

#[test]
fn present_does_not_change_pixels() {
    let mut d = SoftwareDriver::headless(64, 64);
    run_frame(&mut d, |d| d.clear([1.0, 0.0, 0.0, 1.0]));
    let before: Vec<u32> = d.pixels().to_vec();
    d.present();
    assert_eq!(d.pixels(), before.as_slice());
}

#[test]
fn multiple_frames_no_panic() {
    let mut d = SoftwareDriver::headless(32, 32);
    for _ in 0..10 {
        run_frame(&mut d, |d| d.clear([0.0, 0.0, 0.0, 1.0]));
    }
}

// ---------------------------------------------------------------------------
// Clear
// ---------------------------------------------------------------------------

#[test]
fn clear_fills_all_pixels_red() {
    let mut d = SoftwareDriver::headless(16, 16);
    run_frame(&mut d, |d| d.clear([1.0, 0.0, 0.0, 1.0]));
    assert!(d.pixels().iter().all(|&p| p == RED), "not all pixels are red");
}

#[test]
fn clear_black() {
    let mut d = SoftwareDriver::headless(16, 16);
    run_frame(&mut d, |d| d.clear([0.0, 0.0, 0.0, 1.0]));
    assert!(d.pixels().iter().all(|&p| p == BLACK));
}

#[test]
fn clear_white() {
    let mut d = SoftwareDriver::headless(16, 16);
    run_frame(&mut d, |d| d.clear([1.0, 1.0, 1.0, 1.0]));
    assert!(d.pixels().iter().all(|&p| p == WHITE));
}

#[test]
fn clear_transparent() {
    let mut d = SoftwareDriver::headless(16, 16);
    run_frame(&mut d, |d| d.clear([0.0, 0.0, 0.0, 0.0]));
    assert!(d.pixels().iter().all(|&p| p == 0x00_00_00_00));
}

#[test]
fn multiple_clears_last_wins() {
    let mut d = SoftwareDriver::headless(16, 16);
    run_frame(&mut d, |d| {
        d.clear([1.0, 0.0, 0.0, 1.0]); // red
        d.clear([0.0, 0.0, 1.0, 1.0]); // blue — should win
    });
    assert!(d.pixels().iter().all(|&p| p == BLUE));
}

#[test]
fn clear_does_not_bleed_into_next_frame() {
    let mut d = SoftwareDriver::headless(16, 16);
    run_frame(&mut d, |d| d.clear([1.0, 0.0, 0.0, 1.0])); // red frame
    run_frame(&mut d, |d| d.clear([0.0, 1.0, 0.0, 1.0])); // green frame
    assert!(d.pixels().iter().all(|&p| p == GREEN));
}

// ---------------------------------------------------------------------------
// Upload Mesh
// ---------------------------------------------------------------------------

#[test]
fn upload_mesh_returns_handle_zero() {
    let mut d = SoftwareDriver::headless(64, 64);
    d.begin_frame();
    let (v, i) = triangle_mesh();
    assert_eq!(d.upload_mesh(&v, &i), 0);
}

#[test]
fn upload_second_mesh_returns_one() {
    let mut d = SoftwareDriver::headless(64, 64);
    d.begin_frame();
    let (v, i) = triangle_mesh();
    assert_eq!(d.upload_mesh(&v, &i), 0);
    assert_eq!(d.upload_mesh(&v, &i), 1);
}

#[test]
fn upload_empty_mesh_no_panic() {
    let mut d = SoftwareDriver::headless(64, 64);
    d.begin_frame();
    let _ = d.upload_mesh(&[], &[]);
}

#[test]
fn handles_recycled_after_begin_frame() {
    let mut d = SoftwareDriver::headless(64, 64);
    let (v, i) = triangle_mesh();

    d.begin_frame();
    assert_eq!(d.upload_mesh(&v, &i), 0);
    d.end_frame();
    d.present();

    d.begin_frame(); // pool clears
    assert_eq!(d.upload_mesh(&v, &i), 0, "handle should reset to 0 after begin_frame");
}

// ---------------------------------------------------------------------------
// Draw Mesh — Triangle Rasterisation
// These tests require the software rasterizer to be implemented.
// ---------------------------------------------------------------------------

#[test]
fn draw_triangle_centroid_pixel() {
    let (w, h) = (100, 100);
    let mut d = SoftwareDriver::headless(w, h);

    run_frame(&mut d, |d| {
        d.clear([0.0, 0.0, 0.0, 1.0]);
        let verts = vec![
            Vertex { position: [ 0.0,  0.6], color: [1.0, 1.0, 1.0, 1.0] },
            Vertex { position: [-0.6, -0.6], color: [1.0, 1.0, 1.0, 1.0] },
            Vertex { position: [ 0.6, -0.6], color: [1.0, 1.0, 1.0, 1.0] },
        ];
        let h_mesh = d.upload_mesh(&verts, &[0, 1, 2]);
        d.draw_mesh(h_mesh, Mat3::IDENTITY, [1.0, 1.0, 1.0, 1.0]);
    });

    // Centroid ≈ (0, -0.2) → pixel (~50, ~60)
    let centroid_idx = clip_to_pixel(0.0, -0.2, w, h);
    assert_eq!(d.pixels()[centroid_idx], WHITE, "centroid should be white");

    // Corner far from triangle should remain black
    let outside_idx = clip_to_pixel(-0.9, 0.9, w, h);
    assert_eq!(d.pixels()[outside_idx], BLACK, "outside pixel should be black");
}

#[test]
fn draw_triangle_vertex_colours_interpolated() {
    let (w, h) = (100, 100);
    let mut d = SoftwareDriver::headless(w, h);

    run_frame(&mut d, |d| {
        d.clear([0.0, 0.0, 0.0, 1.0]);
        let verts = vec![
            Vertex { position: [ 0.0,  0.8], color: [1.0, 0.0, 0.0, 1.0] }, // top — red
            Vertex { position: [-0.8, -0.8], color: [0.0, 1.0, 0.0, 1.0] }, // bottom-left — green
            Vertex { position: [ 0.8, -0.8], color: [0.0, 0.0, 1.0, 1.0] }, // bottom-right — blue
        ];
        let h_mesh = d.upload_mesh(&verts, &[0, 1, 2]);
        d.draw_mesh(h_mesh, Mat3::IDENTITY, [1.0, 1.0, 1.0, 1.0]);
    });

    fn channel(pixel: u32, shift: u32) -> f32 {
        ((pixel >> shift) & 0xFF) as f32 / 255.0
    }

    // Near top vertex: red channel dominant
    let top_idx = clip_to_pixel(0.0, 0.75, w, h);
    let top = d.pixels()[top_idx];
    assert!(channel(top, 16) >= 0.7, "top pixel should be mostly red");
    assert!(channel(top,  8) <= 0.3, "top pixel should have little green");
    assert!(channel(top,  0) <= 0.3, "top pixel should have little blue");
}

#[test]
fn draw_fills_triangle_interior_not_exterior() {
    let (w, h) = (100, 100);
    let mut d = SoftwareDriver::headless(w, h);

    run_frame(&mut d, |d| {
        d.clear([0.0, 0.0, 0.0, 1.0]);
        // Upper-right quadrant triangle
        let verts = vec![
            Vertex { position: [0.0, 0.0], color: [1.0, 1.0, 1.0, 1.0] },
            Vertex { position: [1.0, 0.0], color: [1.0, 1.0, 1.0, 1.0] },
            Vertex { position: [0.0, 1.0], color: [1.0, 1.0, 1.0, 1.0] },
        ];
        let h_mesh = d.upload_mesh(&verts, &[0, 1, 2]);
        d.draw_mesh(h_mesh, Mat3::IDENTITY, [1.0, 1.0, 1.0, 1.0]);
    });

    let inside_idx  = clip_to_pixel(0.3, 0.3, w, h);
    let outside_idx = clip_to_pixel(-0.5, -0.5, w, h);
    assert_eq!(d.pixels()[inside_idx],  WHITE, "inside triangle should be white");
    assert_eq!(d.pixels()[outside_idx], BLACK, "outside triangle should be black");
}

#[test]
fn draw_triangle_with_translation() {
    let (w, h) = (100, 100);
    let mut d = SoftwareDriver::headless(w, h);

    run_frame(&mut d, |d| {
        d.clear([0.0, 0.0, 0.0, 1.0]);
        let verts = vec![
            Vertex { position: [ 0.0,  0.25], color: [1.0, 1.0, 1.0, 1.0] },
            Vertex { position: [-0.25, -0.25], color: [1.0, 1.0, 1.0, 1.0] },
            Vertex { position: [ 0.25, -0.25], color: [1.0, 1.0, 1.0, 1.0] },
        ];
        let h_mesh = d.upload_mesh(&verts, &[0, 1, 2]);

        // Draw 1: centred at origin
        d.draw_mesh(h_mesh, Mat3::IDENTITY, [1.0, 1.0, 1.0, 1.0]);

        // Draw 2: shifted right by 0.5 clip units
        d.draw_mesh(
            h_mesh,
            Mat3::from_translation(Vec2::new(0.5, 0.0)),
            [1.0, 1.0, 1.0, 1.0],
        );
    });

    let centroid1 = clip_to_pixel(0.0,  0.0, w, h);
    let centroid2 = clip_to_pixel(0.5, 0.0, w, h);
    assert_eq!(d.pixels()[centroid1], WHITE, "first instance centroid should be white");
    assert_eq!(d.pixels()[centroid2], WHITE, "second instance centroid should be white");

    // The gap between them (x ≈ 0.25, well between the two centroids) is black
    let gap = clip_to_pixel(0.25, 0.6, w, h); // above both triangles
    assert_eq!(d.pixels()[gap], BLACK, "gap above triangles should be black");
}

#[test]
fn draw_triangle_with_tint() {
    let (w, h) = (100, 100);
    let mut d = SoftwareDriver::headless(w, h);

    run_frame(&mut d, |d| {
        d.clear([0.0, 0.0, 0.0, 1.0]);
        // White triangle
        let verts = vec![
            Vertex { position: [ 0.0,  0.5], color: [1.0, 1.0, 1.0, 1.0] },
            Vertex { position: [-0.5, -0.5], color: [1.0, 1.0, 1.0, 1.0] },
            Vertex { position: [ 0.5, -0.5], color: [1.0, 1.0, 1.0, 1.0] },
        ];
        let h_mesh = d.upload_mesh(&verts, &[0, 1, 2]);
        // Red tint
        d.draw_mesh(h_mesh, Mat3::IDENTITY, [1.0, 0.0, 0.0, 1.0]);
    });

    let centroid = clip_to_pixel(0.0, -0.15, w, h);
    assert_eq!(d.pixels()[centroid], RED, "tinted centroid should be red");
}

#[test]
fn clear_and_draw_background_preserved() {
    let (w, h) = (100, 100);
    let mut d = SoftwareDriver::headless(w, h);

    run_frame(&mut d, |d| {
        d.clear([0.0, 1.0, 0.0, 1.0]); // green background
        let verts = vec![
            Vertex { position: [ 0.0,  0.2], color: [1.0, 1.0, 1.0, 1.0] },
            Vertex { position: [-0.2, -0.2], color: [1.0, 1.0, 1.0, 1.0] },
            Vertex { position: [ 0.2, -0.2], color: [1.0, 1.0, 1.0, 1.0] },
        ];
        let h_mesh = d.upload_mesh(&verts, &[0, 1, 2]);
        d.draw_mesh(h_mesh, Mat3::IDENTITY, [1.0, 1.0, 1.0, 1.0]);
    });

    let inside_idx  = clip_to_pixel(0.0, 0.0, w, h);
    let outside_idx = clip_to_pixel(-0.9, 0.9, w, h);
    assert_eq!(d.pixels()[inside_idx],  WHITE, "triangle centroid should be white");
    assert_eq!(d.pixels()[outside_idx], GREEN, "background should remain green");
}

#[test]
fn draw_multiple_meshes_in_one_frame() {
    let (w, h) = (100, 100);
    let mut d = SoftwareDriver::headless(w, h);

    run_frame(&mut d, |d| {
        d.clear([0.0, 0.0, 0.0, 1.0]);

        // Triangle A — upper left
        let va = vec![
            Vertex { position: [-0.8,  0.8], color: [1.0, 0.0, 0.0, 1.0] },
            Vertex { position: [-0.4,  0.8], color: [1.0, 0.0, 0.0, 1.0] },
            Vertex { position: [-0.6,  0.4], color: [1.0, 0.0, 0.0, 1.0] },
        ];
        let ha = d.upload_mesh(&va, &[0, 1, 2]);
        d.draw_mesh(ha, Mat3::IDENTITY, [1.0, 1.0, 1.0, 1.0]);

        // Triangle B — lower right
        let vb = vec![
            Vertex { position: [ 0.4, -0.4], color: [0.0, 0.0, 1.0, 1.0] },
            Vertex { position: [ 0.8, -0.4], color: [0.0, 0.0, 1.0, 1.0] },
            Vertex { position: [ 0.6, -0.8], color: [0.0, 0.0, 1.0, 1.0] },
        ];
        let hb = d.upload_mesh(&vb, &[0, 1, 2]);
        d.draw_mesh(hb, Mat3::IDENTITY, [1.0, 1.0, 1.0, 1.0]);
    });

    let ca = clip_to_pixel(-0.6, 0.67, w, h);
    let cb = clip_to_pixel( 0.6, -0.53, w, h);
    assert_eq!(d.pixels()[ca], RED,  "triangle A centroid should be red");
    assert_eq!(d.pixels()[cb], BLUE, "triangle B centroid should be blue");
}
