//! `scene::cross_product3` (0x465551): the cross product of the three floats at esi and at
//! edi into ebp's: x = a.y b.z - a.z b.y, y = a.z b.x - a.x b.z, z = a.x b.y - a.y b.x, each
//! rounded as it is written (so an output over an input is read back as the original
//! does).
//!
//! In:  esi a, edi b, ebp the result
//! Out: every register kept
//!
//! See `scene/vector3.rs`. 16 callers (the marks' tyre marks and their kin). The original's
//! instructions (`fsubp` as its bytes). Refuses a null pointer. Self-test: the original and
//! the Rust version on vectors by lot, the result apart and over each input.
//!
//! Converted 2026-10-21: the hook's Rust version is `cross_product3_plain`, through
//! `rc2k_core::scene::matrices::cross3` (plain floats; the x87 version stays for the Rust
//! callers in unconverted code). Shadow mode and the self-test compare within
//! `service::floats`' tolerances (the self-test at both test precisions, exact at double).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::Regs;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::floats::*;

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

/// The converted version (plain floats): the hook's.
// Converted: `rc2k_core::scene::matrices::cross3`
pub unsafe fn cross_product3_plain(a: u32, b: u32, out: u32) -> Result<(), Refused> {
    if a == 0 || b == 0 || out == 0 {
        return Err(BadInput("a null vector").into());
    }
    let p = &mut unsafe { crate::scene::matrix::GamePlaces::new() };
    rc2k_core::scene::matrices::cross3(p, a, b, out);
    Ok(())
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { cross_product3_plain(r.esi, r.edi, r.ebp) }?;
    Ok(left_of(r))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}
