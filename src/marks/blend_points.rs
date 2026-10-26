//! `marks::blend_points` (0x4BA5E2): three points of whole numbers (eax, ebx, ecx) blended
//! by the weights of entry `dl` of the spline file (`g_marksSplineFile`, 12 bytes each:
//! c's weight, then b's, then a's): each of x, y, z a ka + (b kb + c kc), left unrounded in
//! st(2) .. st(0) (x deepest).
//!
//! In:  `a`, `b`, `c` the points, `entry` the entry
//! Out: x, y, z (the original's st(2) .. st(0))
//!
//! Caller: `sub_4BA646`.
//!
//! the x87 version stays for the Rust callers in unconverted code).

use crate::errors::Refused;
use crate::marks::splines::SPLINE_HEAD;
use crate::math::x87;
use crate::pointers::GamePtr;

/// The Rust version (see the header): the three values (x, y, z).
pub fn blend_points(a: u32, b: u32, c: u32, entry: u8) -> Result<[[u8; 10]; 3], Refused> {
    Ok({
        let at = 12 * entry as usize;
        let file = &SPLINE_HEAD.file;
        let k: [u32; 3] = [file.get_u32(at + 8), file.get_u32(at + 4), file.get_u32(at)];
        let p = |r: u32, c: u32| GamePtr::new(r + 4 * c).read::<i32>("a mark's field");
        crate::marks::try_from_fn(|n| {
            let n = n as u32;
            Ok(x87::blend3_80(p(a, n)?, p(b, n)?, p(c, n)?, k))
        })?
    })
}
