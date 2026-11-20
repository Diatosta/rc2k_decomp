//! The HUD's quad record and sprites: the record `HudBlock.quad.alpha` (alpha, left, top, right,
//! bottom, colour, then the four corners' texture places as whole numbers, << 16 of a texel) made
//! the four vertices of a polygon.
//!
//! Intermediates are `f64`, rounded to `f32` where stored, as in `math::geometry`.

use crate::math::geometry::fistp64;

/// Indices of the record's dwords.
pub const ALPHA: usize = 0;
pub const LEFT: usize = 1;
pub const TOP: usize = 2;
pub const RIGHT: usize = 3;
pub(crate) const BOTTOM: usize = 4;
pub const COLOUR: usize = 5;
/// The corners' texture places: u, v of each corner from here.
pub const PLACES: usize = 6;
pub const RECORD: usize = 14;

/// The places of a texture's rectangle (left, top, width, height in texels): each corner's (u, v),
/// in the corners' order (left top, right top, right bottom, left bottom).
pub fn sprite_places([left, top, w, h]: [u32; 4]) -> [[u32; 2]; 4] {
    let [u0, u1] = [left << 16, left.wrapping_add(w) << 16];
    let [v0, v1] = [top << 16, top.wrapping_add(h) << 16];
    [[u0, v0], [u1, v0], [u1, v1], [u0, v1]]
}

/// The absolute value of a signed number.
pub fn positive(v: u32) -> u32 {
    if (v as i32) < 0 { v.wrapping_neg() } else { v }
}

/// A sprite's corners at (x, y), its size made positive: left, top, right, bottom.
pub fn sprite_corners(x: u32, y: u32, w: u32, h: u32) -> [u32; 4] {
    [
        x,
        y,
        positive(w).wrapping_add(x),
        positive(h).wrapping_add(y),
    ]
}

/// A scaled sprite's width or height (before it is made positive).
pub fn scaled_size(size: u32, scale: f64) -> u32 {
    fistp64(size as i32 as f64 * scale) as u32
}

/// `add_solid_quad`'s texel: every corner's (u, v) on one texel (0xF8 << 16, 8 << 16), a solid
/// colour.
pub const SOLID_TEXEL: [u32; 2] = [0xF8_0000, 0x8_0000];

/// One vertex of the polygon (x at +0, y +4, colour +0x10, u +0x18, v
/// +0x1C; the rest left).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vertex {
    pub x: f32,
    pub y: f32,
    pub colour: u32,
    pub u: f32,
    pub v: f32,
}

/// `i / a / b`, with `i` converted to a float first.
fn int_over_over(i: u32, a: f32, b: f32) -> f32 {
    (i as i32 as f64 / a as f64 / b as f64) as f32
}

/// `add_quad`: the record's four vertices (left top, right top, right bottom, left bottom), the
/// places over `a` and over `b`, the colour's low 24 bits under the alpha; also the colour and
/// its low 24 bits.
pub fn quad_vertices(q: &[u32; RECORD], a: f32, b: f32) -> ([Vertex; 4], u32, u32) {
    let int = |n: usize| q[n] as i32 as f32;
    let corners = [
        (int(LEFT), int(TOP)),
        (int(RIGHT), int(TOP)),
        (int(RIGHT), int(BOTTOM)),
        (int(LEFT), int(BOTTOM)),
    ];
    let low = q[COLOUR] & 0xFF_FFFF;
    let colour = (q[ALPHA] << 24) | low;
    let vertices = std::array::from_fn(|n| Vertex {
        x: corners[n].0,
        y: corners[n].1,
        colour,
        u: int_over_over(q[PLACES + 2 * n], a, b),
        v: int_over_over(q[PLACES + 2 * n + 1], a, b),
    });
    (vertices, colour, low)
}

/// `add_polygon`'s scaling: `v` times `i` over `k` (the virtual screen to the real one).
pub(crate) fn screen_scaled(v: f32, i: u32, k: f32) -> f32 {
    (v as f64 * i as i32 as f64 / k as f64) as f32
}

/// The markers' place along the top (`draw_marker`, `draw_markers_1`): `share` over `of` times `k`
/// plus `c`, rounded.
pub fn marker_x(share: f32, of: f32, k: f32, c: f32) -> u32 {
    fistp64(share as f64 / of as f64 * k as f64 + c as f64) as u32
}

/// `i` over `k`: a scale the markers and the split layouts hand on (kept at 80 bits).
pub fn int_over(i: u32, k: f32) -> f64 {
    i as i32 as f64 / k as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sprites() {
        assert_eq!(
            sprite_places([2, 3, 4, 5]),
            [
                [0x2_0000, 0x3_0000],
                [0x6_0000, 0x3_0000],
                [0x6_0000, 0x8_0000],
                [0x2_0000, 0x8_0000],
            ]
        );
        assert_eq!(positive(-5i32 as u32), 5);
        assert_eq!(positive(0x8000_0000), 0x8000_0000);
        assert_eq!(sprite_corners(10, 20, -4i32 as u32, 6), [10, 20, 14, 26]);
        assert_eq!(scaled_size(24, 2.5), 60);
        assert_eq!(scaled_size(25, 0.5), 12);
        assert_eq!(scaled_size(27, 0.5), 14);
        assert_eq!(scaled_size(-24i32 as u32, 1.5), -36i32 as u32);
        assert_eq!(scaled_size(1, f64::NAN), 0x8000_0000);
    }

    #[test]
    fn vertices() {
        let mut q = [0u32; RECORD];
        q[..6].copy_from_slice(&[0x80, 10, 20, 30, 40, 0xAB12_3456]);
        for n in 0..8 {
            q[PLACES + n] = (n as u32) << 16;
        }
        let (v, colour, low24) = quad_vertices(&q, 256.0, 1.0);
        assert_eq!((colour, low24), (0x8012_3456, 0x12_3456));
        assert_eq!(
            (v[0].x, v[0].y, v[2].x, v[2].y, v[3].x),
            (10.0, 20.0, 30.0, 40.0, 10.0)
        );
        assert_eq!((v[1].u, v[1].v, v[3].v), (512.0, 768.0, 1792.0));
        assert_eq!(v[2].colour, colour);
        assert_eq!(screen_scaled(320.0, 1024, 640.0), 512.0);
        assert_eq!(marker_x(500.0, 1000.0, 560.0, 40.0), 320);
        assert_eq!(marker_x(1.0, 0.0, 1.0, 0.0), 0x8000_0000);
        assert_eq!(int_over(0x78, 256.0), 0.46875);
    }
}
