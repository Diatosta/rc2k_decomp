//! `scene::square_matrix` (0x420E94): a turning matrix's three rows made one long and at
//! right angles to each other again.
//!
//! In:  esi = the matrix (changed in place)
//! Out: every register preserved
//!
//! In the work rows `unk_5C6FD0`, `unk_5C6FE0`, `unk_5C6FF0`: the third = the matrix's
//! third row as a unit vector; the first = the matrix's second row crossed with it, as
//! a unit vector; the second = the third crossed with the first. Then sixteen dwords
//! from `unk_5C6FD0` on go to the matrix: the three rows with whatever their fourth
//! dwords hold, and the four dwords after them as the fourth row.
//! Four callers.
//! Refuses a null matrix.
//! Self-test: matrices by lot.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::scene::matrix::{MATRIX_LEN, test_floats};
use crate::scene::view_matrix::{self, squaring_regions};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use std::cell::Cell;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::square_matrix", rva(0x420E94), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- hook ----

fn rust(r: &Regs) -> Result<Left, Refused> {
    if r.esi == 0 {
        return Err(BadInput("a null matrix").into());
    }
    unsafe { view_matrix::square_matrix(r.esi as usize) };
    Ok(Left::nothing())
}

fn regions(matrix: u32) -> Vec<Region> {
    let mut regions = squaring_regions().to_vec();
    regions.push((matrix, MATRIX_LEN));
    regions
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = if r.esi == 0 {
        Vec::new()
    } else {
        regions(r.esi)
    };
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let work: Cell<[u32; 16]> = Default::default();
    let regions = regions(work.as_ptr() as u32);
    let saved = capture(&squaring_regions());
    let mut seed = 0x5A69_7887u32;
    for case in 0..60 {
        let mut floats = test_floats(&mut seed);
        if case % 11 == 0 {
            // A third row of no length.
            floats[8..11].copy_from_slice(&[0, 0, 0x8000_0000]);
        }
        check_state(
            &mut t,
            &format!("case {case}: {floats:x?}"),
            call_original,
            Regs {
                esi: work.as_ptr() as u32,
                ..Regs::SENTINEL
            },
            &|| {
                restore(&squaring_regions(), &vec![0xAA; saved.len()]);
                work.set(floats);
            },
            &regions,
            &rust,
        );
    }
    restore(&squaring_regions(), &saved);
    t.finish()
}
