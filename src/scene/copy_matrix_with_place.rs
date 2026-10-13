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

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::scene::matrix::{self, MATRIX_LEN, test_floats};
use crate::screens::harness::Left;
use crate::service::harness::{Region, check_state, dispatch_state};
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
    Ok(unsafe { matrix::copy_matrix_with_place(r.esi as usize, r.edi as usize, r.ebp as usize) })
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions: &[Region] = if r.esi == 0 || r.edi == 0 || r.ebp == 0 {
        &[]
    } else {
        &[(r.ebp, MATRIX_LEN)]
    };
    dispatch_state(&HOOK, r, call_original, regions, &rust);
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
    for case in 0..60 {
        let floats = [
            test_floats(&mut seed),
            test_floats(&mut seed),
            test_floats(&mut seed),
        ];
        // The result apart, in the first's place, in the second's.
        let out = [2usize, 0, 1][case % 3];
        let regions: [Region; 3] = [0, 1, 2].map(|n| (address(n), MATRIX_LEN));
        check_state(
            &mut t,
            &format!("case {case}, the result in matrix {out}"),
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
            &rust,
        );
    }
    t.finish()
}
