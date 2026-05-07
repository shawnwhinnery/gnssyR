use glam::{Mat3, Vec2};

/// Scale matrix applied by GPU/CPU drivers so logical square NDC (−1..1 in
/// both axes) maps to a non-stretched centered region on a rectangular surface.
///
/// Matches `gfx-wgpu` and `gfx-software` `end_frame` projection.
pub fn aspect_projection(width: u32, height: u32) -> Mat3 {
    let w = width.max(1) as f32;
    let h = height.max(1) as f32;
    if w > h {
        Mat3::from_scale(Vec2::new(h / w, 1.0))
    } else if h > w {
        Mat3::from_scale(Vec2::new(1.0, w / h))
    } else {
        Mat3::IDENTITY
    }
}

/// Maps a cursor from **full-framebuffer** NDC (x and y each span −1..1 over
/// width and height) into **logical** NDC — the space paths use before
/// [`aspect_projection`] is applied at rasterization.
///
/// Without this, aim vectors built from differences against draw-space NDC
/// skew away from the true on-screen direction off the cardinal axes on
/// non-square surfaces, because pointer coordinates span a stretched square
/// while drawn geometry is aspect-corrected.
pub fn window_ndc_to_logical_ndc(width: u32, height: u32, window_ndc: Vec2) -> Vec2 {
    let w = width.max(1) as f32;
    let h = height.max(1) as f32;
    if w > h {
        Vec2::new(window_ndc.x * (w / h), window_ndc.y)
    } else if h > w {
        Vec2::new(window_ndc.x, window_ndc.y * (h / w))
    } else {
        window_ndc
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn square_viewport_is_identity() {
        assert_eq!(
            aspect_projection(512, 512),
            Mat3::IDENTITY,
        );
        let p = Vec2::new(0.25, -0.5);
        assert_eq!(window_ndc_to_logical_ndc(512, 512, p), p);
    }

    #[test]
    fn wide_viewport_inverse_matches_aspect_scale() {
        let w = 800u32;
        let h = 400u32;
        let proj = aspect_projection(w, h);
        let window = Vec2::new(1.0, 0.25);
        let logical = window_ndc_to_logical_ndc(w, h, window);
        let via_mat = proj.inverse().transform_point2(window);
        assert!((logical - via_mat).length() < 1e-5);
    }

    #[test]
    fn tall_viewport_inverse_matches_aspect_scale() {
        let w = 400u32;
        let h = 800u32;
        let proj = aspect_projection(w, h);
        let window = Vec2::new(-0.5, -1.0);
        let logical = window_ndc_to_logical_ndc(w, h, window);
        let via_mat = proj.inverse().transform_point2(window);
        assert!((logical - via_mat).length() < 1e-5);
    }
}
