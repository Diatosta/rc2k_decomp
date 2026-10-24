//! `physics::multiply_parts` (0x420FD5): the three floats at ebx times those at ecx, part
//! by part, into edx (`fld; fmul; fstp`, each stored before the next is read).
//!
//! In:  ebx, ecx the two vectors, edx where the result goes (may be either)
//! Out: every register kept; no flags read
//!
//! Caller: `sub_45F17E` (0x45F256).

use crate::math::x87;

/// The Rust version.
pub unsafe fn multiply_parts(a: u32, b: u32, out: u32) {
    unsafe {
        for k in 0..3 {
            let x = ((a + 4 * k) as *const u32).read_unaligned();
            let y = ((b + 4 * k) as *const u32).read_unaligned();
            ((out + 4 * k) as *mut u32).write_unaligned(x87::mul(x, y));
        }
    }
}
