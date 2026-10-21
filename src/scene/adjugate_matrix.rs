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

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::math::x87;
use crate::scene::matrix::{MATRIX_LEN, test_floats};
use crate::screens::harness::Left;
use crate::service::floats::{
    FLOAT, Floats, at_test_precisions, check_state_floats, dispatch_state_floats,
};
use crate::service::harness::Region;
use std::cell::Cell;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::adjugate_matrix", rva(0x4BCF6B), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

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

extern "C" fn dispatch(r: &mut Regs) {
    let regions: Vec<Region> = if r.esi != 0 && r.edi != 0 {
        vec![(r.edi, MATRIX_LEN)]
    } else {
        Vec::new()
    };
    let places: Vec<_> = regions.iter().map(|&r| (r, FLOAT)).collect();
    let floats = Floats {
        memory: &places,
        regs: &[],
    };
    dispatch_state_floats(&HOOK, r, call_original, &regions, &floats, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let work: [Cell<[u32; 16]>; 2] = Default::default();
    let address = |n: usize| work[n].as_ptr() as u32;
    let regions = [(address(0), MATRIX_LEN), (address(1), MATRIX_LEN)];
    let mut seed = 0x5E1F_A3C7u32;
    let places = regions.map(|r| (r, FLOAT));
    let tolerant = Floats {
        memory: &places,
        regs: &[],
    };
    for case in 0..60 {
        let floats = [test_floats(&mut seed), [0xAAAA_AAAA; 16]];
        let out = case % 2;
        at_test_precisions(&tolerant, &mut |word, held| {
            check_state_floats(
                &mut t,
                "scene::adjugate_matrix",
                &format!(
                    "control word {word:#x}, case {case}: {:x?}, the result in {out}",
                    floats[0]
                ),
                call_original,
                Regs {
                    esi: address(0),
                    edi: address(out),
                    ..Regs::SENTINEL
                },
                &|| {
                    for (cell, floats) in work.iter().zip(floats) {
                        cell.set(floats);
                    }
                },
                &regions,
                held,
                &rust,
            );
        });
    }
    t.finish()
}
