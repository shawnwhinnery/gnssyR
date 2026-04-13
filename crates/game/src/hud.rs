/// Screen-space HUD overlay — positions are in NDC, not world space.
///
/// Characters are rendered as a classic 7-segment LCD display.
///
///   ─ A ─
///  F     B
///   ─ G ─
///  E     C
///   ─ D ─
///
/// Segment bitmask: bit 6 = A (top) … bit 0 = G (middle)
///
///   A  B  C  D  E  F  G
///   6  5  4  3  2  1  0
use gfx::{
    Color, Vec2,
    shape::line,
    style::{LineCap, LineJoin, Stroke, Style},
    tessellate,
};
use glam::Mat3;

// ---------------------------------------------------------------------------
// Glyph table  (digits 0-9 then letters C G P U)
// ---------------------------------------------------------------------------

#[rustfmt::skip]
const GLYPHS: &[(char, u8)] = &[
    ('0', 0b1111110),  // A B C D E F
    ('1', 0b0110000),  //   B C
    ('2', 0b1101101),  // A B   D E   G
    ('3', 0b1111001),  // A B C D     G
    ('4', 0b0110011),  //   B C     F G
    ('5', 0b1011011),  // A   C D   F G
    ('6', 0b1011111),  // A   C D E F G
    ('7', 0b1110000),  // A B C
    ('8', 0b1111111),  // A B C D E F G
    ('9', 0b1111011),  // A B C D   F G
    ('C', 0b1001110),  // A       E F    (top + left rails + bottom)
    ('G', 0b1011111),  // A   C D E F G  (same shape as 6)
    ('P', 0b1100111),  // A B     E F G  (top + right-upper + left + middle)
    ('U', 0b0111110),  //   B C D E F    (left + right + bottom, no top)
];

// ---------------------------------------------------------------------------
// Display geometry (NDC units)
// ---------------------------------------------------------------------------

const DIGIT_W:   f32 = 0.028;
const DIGIT_H:   f32 = 0.056;
const DIGIT_GAP: f32 = 0.007;
const SEG_INSET: f32 = 0.003;
const SEG_WIDTH: f32 = 0.006;

const HUD_RIGHT: f32 =  0.93;
const HUD_TOP:   f32 =  0.93;
const ROW_PAD:   f32 =  0.010; // vertical gap between rows

// ---------------------------------------------------------------------------
// Public entry points
// ---------------------------------------------------------------------------

/// Draw FPS (top row) and backend name (row below) in the top-right corner.
pub fn draw_fps(driver: &mut dyn gfx::GraphicsDriver, fps: f32) {
    let n = fps.round().clamp(0.0, 9999.0) as u32;
    draw_integer(driver, n, 4, HUD_RIGHT, HUD_TOP);
}

pub fn draw_backend(driver: &mut dyn gfx::GraphicsDriver, name: &str) {
    let row_top = HUD_TOP - DIGIT_H - ROW_PAD;
    draw_str(driver, name, HUD_RIGHT, row_top);
}

// ---------------------------------------------------------------------------
// Internals
// ---------------------------------------------------------------------------

fn draw_integer(driver: &mut dyn gfx::GraphicsDriver, n: u32, max_digits: usize, right: f32, top: f32) {
    let digits = collect_digits(n, max_digits);
    let y = top - DIGIT_H;
    for (i, d) in digits.iter().enumerate() {
        let x = right - (digits.len() - i) as f32 * (DIGIT_W + DIGIT_GAP);
        if let Some(d) = d {
            draw_glyph(driver, char::from_digit(*d as u32, 10).unwrap_or('0'), x, y);
        }
    }
}

fn draw_str(driver: &mut dyn gfx::GraphicsDriver, s: &str, right: f32, top: f32) {
    let chars: Vec<char> = s.chars().collect();
    let y = top - DIGIT_H;
    for (i, ch) in chars.iter().enumerate() {
        let x = right - (chars.len() - i) as f32 * (DIGIT_W + DIGIT_GAP);
        draw_glyph(driver, *ch, x, y);
    }
}

fn collect_digits(mut n: u32, max: usize) -> Vec<Option<usize>> {
    let mut digits = Vec::with_capacity(max);
    loop {
        digits.push(Some((n % 10) as usize));
        n /= 10;
        if n == 0 { break; }
    }
    digits.reverse();
    let pad = max.saturating_sub(digits.len());
    let mut result = vec![None; pad];
    result.extend(digits);
    result
}

fn glyph_mask(ch: char) -> Option<u8> {
    let upper = ch.to_ascii_uppercase();
    GLYPHS.iter().find(|(c, _)| *c == upper).map(|(_, m)| *m)
}

fn draw_glyph(driver: &mut dyn gfx::GraphicsDriver, ch: char, x: f32, y: f32) {
    let Some(mask) = glyph_mask(ch) else { return };

    let w      = DIGIT_W;
    let h      = DIGIT_H;
    let half_h = h / 2.0;
    let ins    = SEG_INSET;
    let style  = Style::stroked(Stroke {
        color: Color::rgba(1.0, 1.0, 1.0, 0.85),
        width: SEG_WIDTH,
        cap:   LineCap::Square,
        join:  LineJoin::Miter,
    });

    if mask & (1 << 6) != 0 { seg(driver, Vec2::new(x + ins, y + h),        Vec2::new(x + w - ins, y + h),        &style); } // A top
    if mask & (1 << 5) != 0 { seg(driver, Vec2::new(x + w,  y + h - ins),   Vec2::new(x + w, y + half_h + ins),   &style); } // B upper-right
    if mask & (1 << 4) != 0 { seg(driver, Vec2::new(x + w,  y + half_h - ins), Vec2::new(x + w, y + ins),         &style); } // C lower-right
    if mask & (1 << 3) != 0 { seg(driver, Vec2::new(x + ins, y),            Vec2::new(x + w - ins, y),            &style); } // D bottom
    if mask & (1 << 2) != 0 { seg(driver, Vec2::new(x,       y + ins),      Vec2::new(x, y + half_h - ins),       &style); } // E lower-left
    if mask & (1 << 1) != 0 { seg(driver, Vec2::new(x,       y + half_h + ins), Vec2::new(x, y + h - ins),        &style); } // F upper-left
    if mask & (1 << 0) != 0 { seg(driver, Vec2::new(x + ins, y + half_h),   Vec2::new(x + w - ins, y + half_h),   &style); } // G middle
}

fn seg(driver: &mut dyn gfx::GraphicsDriver, a: Vec2, b: Vec2, style: &Style) {
    for mesh in tessellate(&line(a, b), style) {
        let handle = driver.upload_mesh(&mesh.vertices, &mesh.indices);
        driver.draw_mesh(handle, Mat3::IDENTITY, [1.0, 1.0, 1.0, 1.0]);
    }
}
