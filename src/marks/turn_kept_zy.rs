//! `marks::turn_kept_zy` (0x4C14D2): the three kept vectors turned: z and y by the first angle's sine and cosine (`marks/kept_turns.rs`).
//!
//! In:  nothing
//! Out: every register kept
//!
//! Caller: `marks::turn_kept_vectors`. Self-test: the inputs by lot, compared on what it writes; put back
//! afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::marks::kept_turns::{FIRST_RVA, turn_kept_zy, vector_dwords};
use crate::marks::leaf_test::test_leaf;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::turn_kept_zy", rva(0x4C14D2), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn written() -> Vec<usize> {
    vector_dwords()
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { turn_kept_zy() };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions: Vec<Region> = written().into_iter().map(at).collect();
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut inputs = vector_dwords();
    inputs.extend(FIRST_RVA);
    test_leaf(
        &mut t,
        &inputs,
        &written(),
        Regs::SENTINEL,
        10,
        call_original,
        &rust,
    );
    t.finish()
}
