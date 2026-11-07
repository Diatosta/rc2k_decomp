//! `effects::blend_points` (0x4BA5E2): three points of whole numbers (eax, ebx, ecx) blended
//! by the weights of entry `dl` of the spline file (`g_marksSplineFile`, 12 bytes each:
//! c's weight, then b's, then a's): each of x, y, z a ka + (b kb + c kc), left unrounded in
//! st(2) .. st(0) (x deepest).
//!
//! Takes: `a`, `b`, `c` the points, `entry` the entry
//! Out: x, y, z (the original's st(2) .. st(0))
//!
//! Caller: `sub_4BA646`.
//!
//! the x87 version stays for the Rust callers in unconverted code).

use crate::effects::paths::Paths;
use crate::math::x87;

/// The Rust version (see the header): the three values (x, y, z).
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
