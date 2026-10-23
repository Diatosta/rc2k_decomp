//! `physics::multiply_parts` (0x420FD5): the three floats at ebx times those at ecx, part
//! by part, into edx (`fld; fmul; fstp`, each stored before the next is read).
//!
//! In:  ebx, ecx the two vectors, edx where the result goes (may be either)
//! Out: every register kept; no flags read
//!
//! Caller: `sub_45F17E` (0x45F256).
//! Self-test: the original on vectors by lot, apart and in the place of each, under
//! three precisions.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

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
