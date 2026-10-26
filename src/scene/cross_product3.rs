//! `scene::cross_product3` (0x465551): the cross product of the three floats at esi and at
//! edi into ebp's: x = a.y b.z - a.z b.y, y = a.z b.x - a.x b.z, z = a.x b.y - a.y b.x, each
//! rounded as it is written (so an output over an input is read back as the original
//! does).
//!
//! In:  esi a, edi b, ebp the result
//! Out: every register kept
//!
//! See `scene/vector3.rs`. 16 callers (the marks' tyre marks and their kin). Refuses a null pointer.
//!
//! Plain Rust since 2026-10-24 (no assembly): the arithmetic is the x87's, in `X`
//! (`math/x87_float.rs`); the original's instructions are kept as `asm_reference` in the tests.

use crate::errors::{BadInput, Refused};
use crate::scene::x87_mem::{fld, fstp};

/// The Rust version (see the header).
pub unsafe fn cross_product3(a: u32, b: u32, out: u32) -> Result<(), Refused> {
    if a == 0 || b == 0 || out == 0 {
        return Err(BadInput("a null vector").into());
    }
    unsafe { cross(a as *const u8, b as *const u8, out as *mut u8) };
    Ok(())
}

/// The three products-differences, each stored as made (`fld; fmul; fld; fmul; fsubp; fstp`).
unsafe fn cross(a: *const u8, b: *const u8, o: *mut u8) {
    fstp(o, 0, fld(a, 4) * fld(b, 8) - fld(a, 8) * fld(b, 4));
    fstp(o, 4, fld(a, 8) * fld(b, 0) - fld(a, 0) * fld(b, 8));
    fstp(o, 8, fld(a, 0) * fld(b, 4) - fld(a, 4) * fld(b, 0));
}

#[cfg(test)]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Kind, differential};

    /// The original's instructions.
    unsafe fn cross_ref(a: *mut u8, b: *mut u8, o: *mut u8) {
        unsafe {
            core::arch::asm!(
                "fld dword ptr [{a} + 4]", "fmul dword ptr [{b} + 8]",
                "fld dword ptr [{a} + 8]", "fmul dword ptr [{b} + 4]",
                ".byte 0xDE, 0xE9", "fstp dword ptr [{o}]",
                "fld dword ptr [{a} + 8]", "fmul dword ptr [{b}]",
                "fld dword ptr [{a}]", "fmul dword ptr [{b} + 8]",
                ".byte 0xDE, 0xE9", "fstp dword ptr [{o} + 4]",
                "fld dword ptr [{a}]", "fmul dword ptr [{b} + 4]",
                "fld dword ptr [{a} + 4]", "fmul dword ptr [{b}]",
                ".byte 0xDE, 0xE9", "fstp dword ptr [{o} + 8]",
                a = in(reg) a,
                b = in(reg) b,
                o = in(reg) o,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
        }
    }

    /// Distinct vectors, then the result over an input.
    #[test]
    fn cross_matches_the_assembly() {
        // dwords 0..3 a, 3..6 b, 6..9 the result, 9 the mode
        differential::<10>(
            "cross_product3",
            5000,
            1,
            [Kind::Float; 10],
            |rng, m| {
                for d in m.iter_mut().take(9) {
                    *d = rng.fl();
                }
                m[9] = rng.below(3) as u32;
            },
            |base| unsafe { run(base, true) },
            |base| unsafe { run(base, false) },
        );
    }

    unsafe fn run(base: *mut u8, reference: bool) {
        unsafe {
            let mode = base.add(36).cast::<u32>().read();
            // 0: apart; 1: the result is a; 2: the result is b
            let o = match mode {
                1 => base,
                2 => base.add(12),
                _ => base.add(24),
            };
            let (a, b) = (base, base.add(12));
            if reference {
                cross_ref(a, b, o)
            } else {
                cross(a, b, o)
            }
        }
    }
}
