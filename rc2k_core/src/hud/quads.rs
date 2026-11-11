//! The HUD's quad record and sprites (from `hud::add_quad` 0x4B72D1,
//! `hud::add_sprite` 0x4B70EE, `hud::add_scaled_sprite` 0x4B716A, `hud::add_solid_quad`
//! 0x4B7216, `hud::add_polygon`'s scaling 0x4B7413, `hud::draw_marker` 0x4B55AB and
//! `hud::draw_markers_1` 0x4B56E3): the record `dword_75BFB0` (alpha, left, top, right,
//! bottom, colour, then the four corners' texture places as whole numbers, << 16 of a
//! texel) made the four vertices of a polygon.
//!
//! As in `math::geometry`, what the x87 keeps on its stack is an `f64` here, rounded to
//! `f32` where the original stores it: the same bits at the game's double precision.

use crate::math::geometry::fistp64;

/// The record's dwords.
pub const ALPHA: usize = 0;
pub const LEFT: usize = 1;
pub const TOP: usize = 2;
pub const RIGHT: usize = 3;
pub const BOTTOM: usize = 4;
pub const COLOUR: usize = 5;
/// The corners' texture places: u, v of each corner from here.
pub const PLACES: usize = 6;
pub const RECORD: usize = 14;

/// The places of a texture's rectangle (left, top, width, height in texels): the record's
/// dwords in the order the original writes them.
pub fn sprite_places([left, top, w, h]: [u32; 4]) -> [(usize, u32); 8] {
    let [u0, u1] = [left << 16, left.wrapping_add(w) << 16];
    let [v0, v1] = [top << 16, top.wrapping_add(h) << 16];
    [
        (6, u0),
        (12, u0),
        (8, u1),
        (10, u1),
        (7, v0),
        (9, v0),
        (11, v1),
        (13, v1),
    ]
}

/// `abs` as the original's `cmp 0; jns; neg`.
pub fn positive(v: u32) -> u32 {
    if (v as i32) < 0 { v.wrapping_neg() } else { v }
}

/// A sprite's corners at (x, y), its size made positive: the record's dwords in the order
/// written.
pub fn sprite_corners(x: u32, y: u32, w: u32, h: u32) -> [(usize, u32); 4] {
    [
        (LEFT, x),
        (RIGHT, positive(w).wrapping_add(x)),
        (TOP, y),
        (BOTTOM, positive(h).wrapping_add(y)),
    ]
}

/// `fild size; fmul scale; fistp`: a scaled sprite's width or height (before it is made
/// positive).
pub fn scaled_size(size: u32, scale: f64) -> u32 {
    fistp64(size as i32 as f64 * scale) as u32
}

/// `add_solid_quad`'s places: every corner on one texel (0xF8 << 16, 8 << 16), a solid
/// colour; the record's dwords in the order written.
pub const SOLID_PLACES: [(usize, u32); 8] = [
    (6, 0xF8_0000),
    (8, 0xF8_0000),
    (10, 0xF8_0000),
    (12, 0xF8_0000),
    (7, 0x8_0000),
    (9, 0x8_0000),
    (11, 0x8_0000),
    (13, 0x8_0000),
];

/// One vertex of the polygon (`g_polygonVertices`: x at +0, y +4, colour +0x10, u +0x18, v
/// +0x1C; the rest left).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vertex {
    pub x: f32,
    pub y: f32,
    pub colour: u32,
    pub u: f32,
    pub v: f32,
}

/// `fild i; fdiv a; fdiv b`.
fn int_over_over(i: u32, a: f32, b: f32) -> f32 {
    (i as i32 as f64 / a as f64 / b as f64) as f32
}

/// `add_quad`: the record's four vertices (left top, right top, right bottom, left bottom),
/// the places over `a` (`flt_75C00C`) and over `b` (`flt_75C010`), the colour's low 24 bits
/// under the alpha; and what is left in eax (the colour) and ebx (the low 24 bits).
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

/// `add_polygon`'s scaling: `fld v; fimul i; fdiv k` (the virtual screen to the real one).
pub fn screen_scaled(v: f32, i: u32, k: f32) -> f32 {
    (v as f64 * i as i32 as f64 / k as f64) as f32
}

/// The markers' place along the top (`draw_marker`, `draw_markers_1`): `fld share; fdiv
/// of; fmul k; fadd c; fistp`.
pub fn marker_x(share: f32, of: f32, k: f32, c: f32) -> u32 {
    fistp64(share as f64 / of as f64 * k as f64 + c as f64) as u32
}

/// `fild i; fdiv k`: a scale the markers and the split layouts hand on (kept at 80 bits).
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
                (6, 0x2_0000),
                (12, 0x2_0000),
                (8, 0x6_0000),
                (10, 0x6_0000),
                (7, 0x3_0000),
                (9, 0x3_0000),
                (11, 0x8_0000),
                (13, 0x8_0000)
            ]
        );
        assert_eq!(positive(-5i32 as u32), 5);
        assert_eq!(positive(0x8000_0000), 0x8000_0000);
        assert_eq!(
            sprite_corners(10, 20, -4i32 as u32, 6),
            [(LEFT, 10), (RIGHT, 14), (TOP, 20), (BOTTOM, 26)]
        );
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
        let (v, eax, ebx) = quad_vertices(&q, 256.0, 1.0);
        assert_eq!((eax, ebx), (0x8012_3456, 0x12_3456));
        assert_eq!(
            (v[0].x, v[0].y, v[2].x, v[2].y, v[3].x),
            (10.0, 20.0, 30.0, 40.0, 10.0)
        );
        assert_eq!((v[1].u, v[1].v, v[3].v), (512.0, 768.0, 1792.0));
        assert_eq!(v[2].colour, eax);
        assert_eq!(screen_scaled(320.0, 1024, 640.0), 512.0);
        assert_eq!(marker_x(500.0, 1000.0, 560.0, 40.0), 320);
        assert_eq!(marker_x(1.0, 0.0, 1.0, 0.0), 0x8000_0000);
        assert_eq!(int_over(0x78, 256.0), 0.46875);
    }
}
