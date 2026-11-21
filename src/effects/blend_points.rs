//! `effects::blend_points`: three points of whole numbers blended by the weights of an
//! entry of the spline file (12 bytes each: c's weight, then b's, then a's): each of x, y,
//! z a ka + (b kb + c kc), left unrounded (x first).

use crate::effects::paths_store::Paths;
use crate::math::x87;

/// The three values (x, y, z).
pub fn blend_points(
    paths: &Paths,
    a: [u32; 3],
    b: [u32; 3],
    c: [u32; 3],
    entry: u8,
) -> [[u8; 10]; 3] {
    let at = 12 * entry as usize;
    let head = paths.spline_head.borrow();
    let word = |o: usize| {
        u32::from_le_bytes([
            head.file[o],
            head.file[o + 1],
            head.file[o + 2],
            head.file[o + 3],
        ])
    };
    let k: [u32; 3] = [word(at + 8), word(at + 4), word(at)];
    std::array::from_fn(|n| x87::blend3_80(a[n] as i32, b[n] as i32, c[n] as i32, k))
}
