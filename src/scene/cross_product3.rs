//! `scene::cross_product3` (0x465551): the cross product of the three floats at esi and at
//! edi into ebp's: x = a.y b.z - a.z b.y, y = a.z b.x - a.x b.z, z = a.x b.y - a.y b.x, each
//! rounded as it is written (so an output over an input is read back as the original
//! does).
//!
//! In:  esi a, edi b, ebp the result
//! Out: every register kept
//!
//! See `scene/vector3.rs`. 16 callers (the marks' tyre marks and their kin). The original's
//! instructions (`fsubp` as its bytes). Refuses a null pointer.
//!
//! Converted 2026-10-21: the Rust version is `cross_product3_plain`, through
//! `rc2k_core::scene::matrices::cross3` (plain floats; the x87 version stays for the Rust
//! callers in unconverted code).

use crate::errors::{BadInput, Refused};

/// The Rust version (see the header).
pub unsafe fn cross_product3(a: u32, b: u32, out: u32) -> Result<(), Refused> {
    if a == 0 || b == 0 || out == 0 {
        return Err(BadInput("a null vector").into());
    }
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
            o = in(reg) out,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
    Ok(())
}
