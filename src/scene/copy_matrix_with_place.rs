//! `scene::copy_matrix_with_place` (0x4271E4): a copy of a matrix, moved by another matrix's place.
//!
//! In:  esi, edi = two matrices, ebp = where the result goes
//! Out: ecx = 0; everything else preserved
//!
//! The sixteen dwords of the first matrix (esi) to the result (ebp), then the first
//! three floats of the second's (edi) fourth row added to the result's.
//! 12 callers.
//! Refuses a null matrix.
//! Self-test: matrices of the test's by lot, also with the result in the place of
//! each of the two.
//!
//! Converted 2026-10-21: the hook's Rust version is `copy_matrix_with_place_plain`, through
//! `rc2k_core::scene::matrices::copy_with_place` (plain floats; the x87 version stays for
//! the Rust callers in unconverted code). Shadow mode and the self-test compare within
//! `service::floats`' tolerances (the self-test at both test precisions, exact at double).

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::scene::matrix::{self, MATRIX_LEN, float_places, test_floats};
use crate::screens::harness::Left;
use crate::service::floats::*;
use crate::service::harness::Region;
use std::cell::Cell;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "scene::copy_matrix_with_place",
    rva(0x4271E4),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- hook ----

fn rust(r: &Regs) -> Result<Left, Refused> {
    if r.esi == 0 || r.edi == 0 || r.ebp == 0 {
        return Err(BadInput("a null matrix").into());
    }
    Ok(unsafe {
        matrix::copy_matrix_with_place_plain(r.esi as usize, r.edi as usize, r.ebp as usize)
    })
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions: &[Region] = if r.esi == 0 || r.edi == 0 || r.ebp == 0 {
        &[]
    } else {
        &[(r.ebp, MATRIX_LEN)]
    };
    let places = float_places(regions);
    let floats = Floats {
        memory: &places,
        regs: &[],
    };
    dispatch_state_floats(&HOOK, r, call_original, regions, &floats, &rust);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // Three matrices of the test's; the functions are given their addresses.
    let work: [Cell<[u32; 16]>; 3] = Default::default();
    let address = |n: usize| work[n].as_ptr() as u32;
    let mut seed = 0x0BAD_5EEDu32;
    let tolerant_places = float_places(&[0, 1, 2].map(|n| (address(n), MATRIX_LEN)));
    let tolerant = Floats {
        memory: &tolerant_places,
        regs: &[],
    };
    at_test_precisions(&tolerant, &mut |word, held| {
        for case in 0..60 {
            let floats = [
                test_floats(&mut seed),
                test_floats(&mut seed),
                test_floats(&mut seed),
            ];
            // The result apart, in the first's place, in the second's.
            let out = [2usize, 0, 1][case % 3];
            let regions: [Region; 3] = [0, 1, 2].map(|n| (address(n), MATRIX_LEN));
            check_state_floats(
                &mut t,
                "scene::copy_matrix_with_place",
                &format!("control word {word:#x}, case {case}, the result in matrix {out}"),
                call_original,
                Regs {
                    esi: address(0),
                    edi: address(1),
                    ebp: address(out),
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
        }
    });
    t.finish()
}
