//! The squares' and windscreen drops' items ending (from
//! `effects::expire_windscreen_flakes` 0x4C06AD and `effects::expire_windscreen_drops` 0x4BCB1D): a live item ends
//! when its cell of the masks (0x80 rows of 0x100, a cell 8 units square) holds the
//! shapes' value, and a windscreen drop also by lot. The place is rounded by `fistp` of a float
//! loaded from memory, which does not depend on the x87's precision.

use crate::math::geometry::fistp;

/// An item's cell: its place rounded, the column (bits 3 .. 10 of x) and the cell's byte
/// in a mask (row bits 3 .. 9 of z, times 0x100, plus the column).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cell {
    pub x: i32,
    pub z: i32,
    pub column: u32,
    pub at: u32,
}

pub fn cell(x: f32, z: f32) -> Cell {
    let (x, z) = (fistp(x), fistp(z));
    let column = ((x as u32) >> 3) & 0xFF;
    Cell {
        x,
        z,
        column,
        at: ((((z as u32) >> 3) & 0x7F) << 8) + column,
    }
}

/// `expire_windscreen_flakes`: a square ends when its cell in the first mask is the shapes' low byte,
/// or (with more than one run of masks, signed) in the second; `mask(n)` mask n's byte of
/// the cell, the second asked only with more runs.
pub fn square_ends(shape: u8, runs: impl FnOnce() -> i32, mask: impl Fn(usize) -> u8) -> bool {
    let first = mask(0) == shape;
    let second = runs() > 1 && mask(1) == shape;
    first || second
}

/// A windscreen drop's draw of the squares' random (`dword_82E2B6`, times 0xC5 plus 0x5D): the
/// random after, and the draw (bits 16 .. 26); a draw of 0 ends it.
pub fn windscreen_drop_draw(random: u32) -> (u32, u32) {
    let r = random.wrapping_mul(0xC5).wrapping_add(0x5D);
    (r, r.rotate_right(16) & 0x7FF)
}

/// Whether a windscreen drop ends by the shapes of the last `steps` steps (signed; the latest
/// first): `value(k)` the shapes' value of step k (`dword_844A10` at the latest step plus
/// k, on a clock of 256), its low byte in either mask's cell (`mask(n)`) ends it. With the
/// last value read (what the original leaves in eax), none with no steps.
pub fn windscreen_drop_hit(
    steps: i32,
    value: impl Fn(u32) -> u32,
    mask: impl Fn(usize) -> u8,
) -> (bool, Option<u32>) {
    let mut last = None;
    for k in (0..steps.max(0) as u32).rev() {
        let v = value(k);
        last = Some(v);
        let al = v as u8;
        if mask(0) == al || mask(1) == al {
            return (true, last);
        }
    }
    (false, last)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cells() {
        let c = cell(17.5, 24.4);
        assert_eq!((c.x, c.z, c.column, c.at), (18, 24, 2, 0x302));
        let c = cell(-8.0, 0x400 as f32);
        assert_eq!((c.column, c.at), (0xFF, 0xFF));
        assert!(square_ends(3, || 0, |_| 3));
        assert!(square_ends(3, || 2, |n| [1, 3][n]));
        assert!(!square_ends(3, || 1, |n| [1, 3][n]));
        assert_eq!(windscreen_drop_draw(0), (0x5D, 0));
        let (hit, last) = windscreen_drop_hit(3, |k| 0x100 + k, |n| [9, 1][n]);
        assert_eq!((hit, last), (true, Some(0x101)));
        assert_eq!(
            windscreen_drop_hit(0, |_| unreachable!(), |_| 0),
            (false, None)
        );
    }
}
