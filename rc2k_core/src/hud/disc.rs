//! The HUD's disc (from `hud::disc_vertex` 0x4B6D10 and
//! `hud::turned_vertex` 0x4B7058): a ring of 0x11 entries (the centre and sixteen round
//! it) made vertices, and the needle's turned vertices. As in `math::vector`, what the
//! x87 keeps on its stack is an `f64`, rounded to `f32` where the original stores it: the
//! same bits at the game's double precision. The disc's ring and triangles
//! (from `hud::draw_disc` 0x4B6C4E) and the needle's turn and corners (from
//! `hud::draw_needle` 0x4B6ECD): the roundings that follow the x87's control word (the
//! texture's turn, the needle's) stay with the caller, passed in or worked out from what these
//! return.

use crate::physics::x87::{below, loaded};

/// The ring's entries.
pub const ENTRIES: u32 = 0x11;

/// One ring entry: its place and its texture place
///.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RingEntry {
    pub x: f32,
    pub y: f32,
    pub u: f32,
    pub v: f32,
}

/// The disc's size and places: its radius, the texture's scale
///, middle, shift and
/// size, and the centre on the screen.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Disc {
    pub radius: f32,
    pub texture_scale: f32,
    pub texture_middle: f32,
    pub texture_shift: [f32; 2],
    pub texture_size: f32,
    pub centre: [f32; 2],
}

/// A vertex's place and texture place, as stored (+0, +4 and +0x18, +0x1C).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DiscVertex {
    pub place: [f32; 2],
    pub texture: [f32; 2],
}

/// `hud::disc_vertex`'s floats: the texture place (u, -v) times the radius times the
/// texture's scale, plus the middle and the shift, over the size; the place (x, -y) times
/// the radius plus the centre.
pub fn vertex(e: RingEntry, d: &Disc) -> DiscVertex {
    let (r, k) = (d.radius as f64, d.texture_scale as f64);
    let texture = |t: f32, n: usize| {
        ((t as f64 * r * k + d.texture_middle as f64 + d.texture_shift[n] as f64)
            / d.texture_size as f64) as f32
    };
    let place = |p: f32, n: usize| (p as f64 * r + d.centre[n] as f64) as f32;
    DiscVertex {
        texture: [texture(e.u, 0), texture(-e.v, 1)],
        place: [place(e.x, 0), place(-e.y, 1)],
    }
}

/// White under the quad record's alpha (its low byte).
pub fn white_under(alpha: u32) -> u32 {
    (alpha << 24) | 0xFF_FFFF
}

/// `hud::turned_vertex`'s texture place: `fld p; fadd shift; fdiv scale`.
pub fn turned_texture(p: f32, shift: f32, scale: f32) -> f32 {
    ((p as f64 + shift as f64) / scale as f64) as f32
}

/// `hud::turned_vertex`'s place: `fld t; fmul size; fadd centre` (the point turned).
pub fn turned_place(t: f32, size: f32, centre: f32) -> f32 {
    (t as f64 * size as f64 + centre as f64) as f32
}

/// The disc's segments.
pub const SEGMENTS: u32 = 0x10;

/// Ring entry k + 1's angles (of 0x1000 a turn) for the sines and cosines: its place's
/// (k times 0x100) and its texture place's (that plus the texture's turn, & 0xFFF).
pub fn ring_angles(turn: u32) -> [[u32; 2]; SEGMENTS as usize] {
    std::array::from_fn(|k| {
        let a = (k as u32) << 8;
        [a, a.wrapping_add(turn) & 0xFFF]
    })
}

/// Triangle k's three ring entries: the centre, k + 1 and k + 2 (wrapping past 0x10).
pub fn disc_triangle(k: u32) -> [u32; 3] {
    let third = if k + 2 > SEGMENTS {
        k + 2 - SEGMENTS
    } else {
        k + 2
    };
    [0, k + 1, third]
}

/// The needle's constants after its least value (`flt_75C2BC` + 4 n, each asked for where
/// it is read): the angles' base and second base, the split, the most with the kind switch,
/// the scale, the second scale, the most without.
pub mod needle {
    pub const BASE: usize = 0;
    pub const BASE_2: usize = 1;
    pub const SPLIT: usize = 2;
    pub const MOST: usize = 3;
    pub const SCALE: usize = 4;
    pub const SCALE_2: usize = 5;
    pub const MOST_2: usize = 6;
}

/// The needle's turn before its two factors and rounding: `(value [- less]) / over +
/// plus`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NeedleTurn {
    pub value: f32,
    pub less: Option<f32>,
    pub over: f32,
    pub plus: f32,
}

/// `hud::draw_needle`'s value held to the least and the most (C0 of the
/// compares: an unordered one counts as below), and which way it is made an angle: with the
/// kind switch over the scale plus the base; without, below the split over the second scale
/// plus the base, else less the split over the scale plus the second base. `value` is the
/// needle's whole number as the x87 made it a float; `k` the constants (`needle`).
pub fn needle_turn(value: f32, kind: bool, least: f32, k: impl Fn(usize) -> f32) -> NeedleTurn {
    use needle::*;
    let held = |most: f32| {
        let mut v = value;
        if below(v as f64, least as f64) {
            v = loaded(least);
        }
        if !below(v as f64, most as f64) {
            v = loaded(most);
        }
        v
    };
    if kind {
        let value = held(k(MOST));
        NeedleTurn {
            value,
            less: None,
            over: k(SCALE),
            plus: k(BASE),
        }
    } else {
        let value = held(k(MOST_2));
        let split = k(SPLIT);
        if below(value as f64, split as f64) {
            NeedleTurn {
                value,
                less: None,
                over: k(SCALE_2),
                plus: k(BASE),
            }
        } else {
            NeedleTurn {
                value,
                less: Some(split),
                over: k(SCALE),
                plus: k(BASE_2),
            }
        }
    }
}

/// The needle's angle: the rounded turn times -16.
pub fn needle_angle(turn: u32) -> u32 {
    (turn << 4).wrapping_neg()
}

/// The needle's four corners (x, y) from its two x and two y, in the order drawn.
pub fn needle_corners([x0, y0, x1, y1]: [u32; 4]) -> [(u32, u32); 4] {
    [(x0, y0), (x1, y0), (x1, y1), (x0, y1)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ring_and_needle() {
        let a = ring_angles(0xF80);
        assert_eq!(
            (a[0], a[1], a[15]),
            ([0, 0xF80], [0x100, 0x080], [0xF00, 0xE80])
        );
        assert_eq!(disc_triangle(0), [0, 1, 2]);
        assert_eq!(disc_triangle(14), [0, 15, 16]);
        assert_eq!(disc_triangle(15), [0, 16, 1]);
        let ks = [0.5f32, 1.25, 100.0, 200.0, 80.0, 40.0, 300.0];
        let k = |n: usize| ks[n];
        let t = needle_turn(5.0, true, 10.0, k);
        assert_eq!((t.value, t.less, t.over, t.plus), (10.0, None, 80.0, 0.5));
        assert_eq!(needle_turn(250.0, true, 10.0, k).value, 200.0);
        let t = needle_turn(50.0, false, 10.0, k);
        assert_eq!((t.value, t.less, t.over, t.plus), (50.0, None, 40.0, 0.5));
        let t = needle_turn(9000.0, false, 10.0, k);
        assert_eq!(
            (t.value, t.less, t.over, t.plus),
            (300.0, Some(100.0), 80.0, 1.25)
        );
        // an unordered value is held to the least (below), then kept (below the most)
        assert_eq!(needle_turn(f32::NAN, true, 10.0, k).value, 10.0);
        assert_eq!(needle_angle(3), 0xFFFF_FFD0);
        assert_eq!(
            needle_corners([1, 2, 3, 4]),
            [(1, 2), (3, 2), (3, 4), (1, 4)]
        );
    }

    #[test]
    fn vertices() {
        let d = Disc {
            radius: 40.0,
            texture_scale: 0.5,
            texture_middle: 128.0,
            texture_shift: [3.0, -2.0],
            texture_size: 256.0,
            centre: [320.0, 240.0],
        };
        let e = RingEntry {
            x: 1.0,
            y: 0.5,
            u: 1.0,
            v: 0.5,
        };
        let v = vertex(e, &d);
        assert_eq!(v.place, [360.0, 220.0]);
        assert_eq!(v.texture, [(20.0 + 131.0) / 256.0, (-10.0 + 126.0) / 256.0]);
        assert_eq!(white_under(0xC1), 0xC1FF_FFFF);
        assert_eq!(turned_texture(12.5, 64.0, 256.0), 76.5 / 256.0);
        assert_eq!(turned_place(-2.0, 1.5, 320.0), 317.0);
    }
}
