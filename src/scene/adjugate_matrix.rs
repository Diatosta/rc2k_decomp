//! `scene::adjugate_matrix` (0x4BCF6B): a matrix's turn undone without the division: its
//! 3 x 3 part's cofactors, transposed (rows 0x10 bytes apart), and its place (+0x30 ..
//! +0x38) negated, not turned.
//!
//! In:  esi the matrix, edi where the result goes (the matrix itself allowed)
//! Out: every register kept
//!
//! Each of the nine a product less a product (`x87::product_difference`), written in the
//! original's order and read from memory as it then is (in place, later ones see earlier
//! results). +0xC, +0x1C, +0x2C, +0x3C are not written. One caller, `sub_4C9AF6`.
//! Refuses a null matrix or result. Self-test: matrices by lot (`scene::matrix`), the
//! result apart and in the matrix's own place, at the test precisions. The hook's Rust
//! version is `adjugate_matrix_plain`; the x87's stays for `marks::cars_particles`.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::Regs;
use crate::math::x87;
use crate::screens::harness::Left;
use crate::service::floats::floats_entry;

/// The Rust version (see the header), the x87's: for `marks::cars_particles`
/// (unconverted).
pub unsafe fn adjugate_matrix(matrix: u32, out: u32) {
    use rc2k_core::scene::matrices::COFACTORS;
    let get = |o: u32| unsafe { ((matrix + o) as *const u32).read_unaligned() };
    let put = |o: u32, v: u32| unsafe { ((out + o) as *mut u32).write_unaligned(v) };
    for (to, [a, b, c, d]) in COFACTORS {
        put(to, x87::product_difference(get(a), get(b), get(c), get(d)));
    }
    for o in [0x30, 0x34, 0x38] {
        put(o, x87::negated(get(o)));
    }
}

/// The converted `adjugate_matrix` (plain floats): the hook's.
// Converted: `rc2k_core::scene::matrices::adjugate_matrix`
pub unsafe fn adjugate_matrix_plain(matrix: u32, out: u32) {
    let mut p = unsafe { crate::scene::matrix::GamePlaces::new() };
    rc2k_core::scene::matrices::adjugate_matrix(&mut p, matrix, out);
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    if r.esi == 0 || r.edi == 0 {
        return Err(BadInput("a null matrix or result").into());
    }
    unsafe { adjugate_matrix_plain(r.esi, r.edi) };
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}
