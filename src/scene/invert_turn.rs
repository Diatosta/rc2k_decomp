//! `scene::invert_turn` (0x420CC2): a turning matrix undone.
//!
//! In:  esi = the matrix, edi = where the result goes
//! Out: every register preserved
//!
//! The matrix's nine floats (three rows of three) are copied to `flt_5C7050` ..; their
//! nine cofactors (each a product less a product, every other one negated) are made at
//! `flt_5C7074` .. and stored the other way round: the result's rows are the cofactors
//! of the matrix's columns. No division by the determinant: `scene::square_matrix` on
//! the result makes its rows one long instead. The result's fourth row and column are
//! what that function copies.
//! Six callers.
//! Refuses a null matrix.
//! Self-test: matrices by lot, the result apart and in the matrix's own place.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::scene::matrix::{MATRIX_LEN, test_floats};
use crate::scene::view_matrix::{self, INVERTED_LEN, INVERTED_RVA, squaring_regions};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use std::cell::Cell;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::invert_turn", rva(0x420CC2), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- hook ----

fn rust(r: &Regs) -> Result<Left, Refused> {
    if r.esi == 0 || r.edi == 0 {
        return Err(BadInput("a null matrix").into());
    }
    unsafe { view_matrix::invert_turn(r.esi as usize, r.edi as usize) };
    Ok(Left::nothing())
}

fn work_regions() -> Vec<Region> {
    let mut regions = squaring_regions().to_vec();
    regions.push((game_addr(INVERTED_RVA) as u32, INVERTED_LEN));
    regions
}

extern "C" fn dispatch(r: &mut Regs) {
    let mut regions = Vec::new();
    if r.esi != 0 && r.edi != 0 {
        regions = work_regions();
        regions.push((r.edi, MATRIX_LEN));
    }
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let work: [Cell<[u32; 16]>; 2] = Default::default();
    let address = |n: usize| work[n].as_ptr() as u32;
    let game = work_regions();
    let saved = capture(&game);
    let mut regions = game.clone();
    regions.extend([(address(0), MATRIX_LEN), (address(1), MATRIX_LEN)]);
    let mut seed = 0x7B8C_9DAEu32;
    for case in 0..60 {
        let floats = [test_floats(&mut seed), test_floats(&mut seed)];
        let out = case % 2;
        check_state(
            &mut t,
            &format!("case {case}: {:x?}, the result in {out}", floats[0]),
            call_original,
            Regs {
                esi: address(0),
                edi: address(out),
                ..Regs::SENTINEL
            },
            &|| {
                restore(&game, &vec![0xAA; saved.len()]);
                for (cell, floats) in work.iter().zip(floats) {
                    cell.set(floats);
                }
            },
            &regions,
            &rust,
        );
    }
    restore(&game, &saved);
    t.finish()
}
