//! `effects::blend_points`: three points of whole numbers blended by the weights of an
//! entry of the spline file (12 bytes each: c's weight, then b's, then a's): each of x, y,
//! z a ka + (b kb + c kc), left unrounded (x first).

use crate::effects::paths::Paths;
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
    let file = &paths.spline_head.file;
    let k: [u32; 3] = [file.get_u32(at + 8), file.get_u32(at + 4), file.get_u32(at)];
    std::array::from_fn(|n| x87::blend3_80(a[n] as i32, b[n] as i32, c[n] as i32, k))
}
