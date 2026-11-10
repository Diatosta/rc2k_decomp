//! The stage's progress bars along the top of the HUD (converted 2026-10-21 from
//! `hud::draw_stage_bar` 0x4B5A86, `hud::draw_stage_bars_1` 0x4B5E29 and
//! `hud::draw_stage_bars_2` 0x4B63C2): a frame of solid quads, then for each part of the
//! stage (`g_overlayPartMiles`) its stretch of the bar across 0x229 pixels, a black border,
//! the part's colour inside under alpha 0x78 and a line between parts. The single layout
//! has one band, the split layouts one a player (two bands, 0xE apart).
//!
//! As in `math::geometry`, what the x87 keeps on its stack is an `f64` here, rounded to
//! `f32` where the original stores it: the same bits at the game's double precision.

use crate::Invalid;
use crate::math::geometry::fistp64;

/// The most parts (`g_overlayPartMiles`, `dword_6D160C`: 0x40 each).
pub const PARTS: u32 = 0x40;
/// The frame's and the lines' colour.
pub const FRAME: u32 = 0x21_2463;
/// The bar's width in pixels (`dword_75C000`).
pub const WIDTH: u32 = 0x229;
/// The last colour offset: the colours are four dwords (`dword_75BF70`) read at a byte
/// offset.
pub const LAST_KIND: u32 = 0xC;
/// The alphas: the frame and borders, the colour inside.
pub const SOLID: u32 = 0xE0;
pub const SEE_THROUGH: u32 = 0x78;

/// Refuses more than `PARTS` parts and a colour offset past the colours (`kind(table, n)`
/// for each player's table), before anything.
pub fn check(count: u32, players: u32, kind: impl Fn(u32, u32) -> u32) -> Result<(), Invalid> {
    if count > PARTS {
        return Err(Invalid("a stage bar of more than 0x40 parts"));
    }
    if (0..players).any(|p| (0..count).any(|n| kind(p, n) > LAST_KIND)) {
        return Err(Invalid("a stage bar colour past dword_75BF70"));
    }
    Ok(())
}

/// The frame's quads (colour, corners), `dy` lower: one band or two.
pub fn frame(dy: u32, two: bool) -> Vec<(u32, [u32; 4])> {
    let y = |v: u32| v.wrapping_add(dy);
    if !two {
        return vec![
            (FRAME, [0, y(0), 0x89, y(0x20)]),
            (FRAME, [0x2AF, y(0), 0x320, y(0x20)]),
            (FRAME, [0x89, y(0), 0x2AF, y(3)]),
            (FRAME, [0x89, y(0x1D), 0x2AF, y(0x20)]),
            (0, [0, y(0x1F), 0x320, y(0x20)]),
        ];
    }
    vec![
        (FRAME, [0, y(0), 0x89, y(0x1E)]),
        (FRAME, [0x2AF, y(0), 0x320, y(0x1E)]),
        (FRAME, [0x89, y(0), 0x2AF, y(2)]),
        (FRAME, [0x89, y(0xE), 0x2AF, y(0x10)]),
        (FRAME, [0x89, y(0x1C), 0x2AF, y(0x1E)]),
        (0, [0, y(0xFFFF_FFFF), 0x320, y(0)]),
        (0, [0, y(0x1E), 0x320, y(0x1F)]),
    ]
}

/// A part's end (`dword_75BFFC`): the miles so far plus its own (`fld; fadd; fstp`), the
/// last part's the record's total.
pub fn part_end(miles: f32, end: f32, last: bool, total: f32) -> f32 {
    if last {
        total
    } else {
        (miles as f64 + end as f64) as f32
    }
}

/// `fild width; fmul miles; fdiv total; fistp`: a place across the bar.
pub fn place(width: u32, miles: f32, total: f32) -> u32 {
    fistp64(width as i32 as f64 * miles as f64 / total as f64) as u32
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Quad {
    pub alpha: Option<u32>,
    pub colour: Option<u32>,
    pub corners: [u32; 4],
    pub eax: u32,
    pub ebx: u32,
    pub edx: Option<u32>,
}

/// A band's rows: its top, and its height to the border's lower row.
fn bands(dy: u32, two: bool) -> Vec<(u32, u32)> {
    if two {
        vec![(dy.wrapping_add(2), 0xB), (dy.wrapping_add(0x10), 0xB)]
    } else {
        vec![(dy.wrapping_add(3), 0x19)]
    }
}

/// A part's quads, its places `a` and `b` (start and end) in, its colours (one a band), in
/// the original's order: each band's border, the colours inside, and unless it is the last
/// part the lines after it.
pub fn part_quads(a: u32, b: u32, colours: &[u32], last: bool, dy: u32) -> Vec<Quad> {
    let two = colours.len() == 2;
    let rows = bands(dy, two);
    let [x0, x1] = [a.wrapping_add(0x89), b.wrapping_add(0x86)];
    let mut out = Vec::new();
    for (k, &(t, h)) in rows.iter().enumerate() {
        let border = |colour: Option<u32>, corners| Quad {
            alpha: (k == 0 && colour.is_some()).then_some(SOLID),
            colour,
            corners,
            eax: x0,
            ebx: x1,
            edx: None,
        };
        out.push(border(Some(0), [x0, t, x1, t.wrapping_add(1)]));
        out.push(border(None, [x1, t, x1.wrapping_add(1), t.wrapping_add(h)]));
        out.push(border(
            None,
            [
                x0,
                t.wrapping_add(1),
                x0.wrapping_add(1),
                t.wrapping_add(h + 1),
            ],
        ));
        out.push(border(
            None,
            [
                x0.wrapping_add(1),
                t.wrapping_add(h),
                x1.wrapping_add(1),
                t.wrapping_add(h + 1),
            ],
        ));
    }
    let [x0, x1] = [x0.wrapping_add(1), x1.wrapping_sub(1)];
    for (k, (&(t, h), &c)) in rows.iter().zip(colours).enumerate() {
        out.push(Quad {
            alpha: (k == 0).then_some(SEE_THROUGH),
            colour: Some(c),
            corners: [x0, t.wrapping_add(1), x1.wrapping_add(1), t.wrapping_add(h)],
            eax: x0,
            ebx: x1,
            edx: Some(c),
        });
    }
    if !last {
        let [x0, x1] = [x1.wrapping_add(2), x1.wrapping_add(3)];
        for (k, &(t, h)) in rows.iter().enumerate() {
            out.push(Quad {
                alpha: (k == 0).then_some(SOLID),
                colour: Some(FRAME),
                corners: [x0, t, x1.wrapping_add(1), t.wrapping_add(h + 1)],
                eax: x0,
                ebx: x1,
                edx: None,
            });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bars() {
        assert!(check(0x41, 1, |_, _| 0).is_err());
        assert!(check(3, 2, |p, n| if p == 1 && n == 2 { 0xD } else { 0 }).is_err());
        assert!(check(3, 1, |p, n| if p == 1 && n == 2 { 0xD } else { 0xC }).is_ok());
        assert_eq!(frame(0, false).len(), 5);
        assert_eq!(frame(0x11E, true)[5].1, [0, 0x11D, 0x320, 0x11E]);
        assert_eq!(part_end(1.5, 2.0, false, 9.0), 3.5);
        assert_eq!(part_end(1.5, 2.0, true, 9.0), 9.0);
        assert_eq!(place(WIDTH, 6.25, 12.5), 276);
        assert_eq!(place(WIDTH, 1.0, 0.0), 0x8000_0000);
        let single = part_quads(0, 0x10, &[7], false, 0);
        assert_eq!(single.len(), 6);
        assert_eq!(single[0].corners, [0x89, 3, 0x96, 4]);
        assert_eq!(single[0].alpha, Some(SOLID));
        assert_eq!(single[1].alpha, None);
        assert_eq!(single[3].corners, [0x8A, 0x1C, 0x97, 0x1D]);
        assert_eq!(
            single[4],
            Quad {
                alpha: Some(SEE_THROUGH),
                colour: Some(7),
                corners: [0x8A, 4, 0x96, 0x1C],
                eax: 0x8A,
                ebx: 0x95,
                edx: Some(7)
            }
        );
        assert_eq!(single[5].corners, [0x97, 3, 0x99, 0x1D]);
        assert_eq!(part_quads(0, 0x10, &[7], true, 0).len(), 5);
        let two = part_quads(0, 0x10, &[7, 8], false, 0);
        assert_eq!(two.len(), 12);
        assert_eq!(two[4].alpha, None);
        assert_eq!(two[4].corners, [0x89, 0x10, 0x96, 0x11]);
        assert_eq!(two[9].corners, [0x8A, 0x11, 0x96, 0x1B]);
        assert_eq!(two[11].corners, [0x97, 0x10, 0x99, 0x1C]);
    }
}
