//! `marks::scale_kept_vectors` (0x4C15F4): three kept vectors scaled by their kept factors
//! (`marks::scale_vectors_by`' pattern): `flt_76927A` .. = `dword_76926E` .. by
//! `dword_76926A`, `flt_7692CE` .. = `dword_7692C2` .. by `dword_7692BE`, `flt_7692B2` ..
//! = `dword_7692A6` .. by `dword_7692A2`.
//!
//! In:  nothing
//! Out: every register kept
//!
//! Caller: `sub_4C1270`. Self-test: the twelve inputs by lot, compared on the nine
//! results; put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::marks::leaf_test::test_leaf;
use crate::marks::scale_vectors_by::{inputs_of, results_of, scale_triples};
use crate::screens::harness::Left;
use crate::service::harness::{at, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::scale_kept_vectors", rva(0x4C15F4), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// (the factor, the vector, the result).
const PAIRS: [(usize, usize, usize); 3] = [
    (rva(0x76926A), rva(0x76926E), rva(0x76927A)),
    (rva(0x7692BE), rva(0x7692C2), rva(0x7692CE)),
    (rva(0x7692A2), rva(0x7692A6), rva(0x7692B2)),
];

/// The Rust version (see the header).
pub unsafe fn scale_kept_vectors() {
    unsafe { scale_triples(&PAIRS) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { scale_kept_vectors() };
    Ok(Left::nothing())
}

/// What it writes.
pub fn regions() -> Vec<crate::service::harness::Region> {
    results_of(&PAIRS).into_iter().map(at).collect()
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let (inputs, written) = (inputs_of(&PAIRS), results_of(&PAIRS));
    test_leaf(
        &mut t,
        &inputs,
        &written,
        Regs::SENTINEL,
        8,
        call_original,
        &rust,
    );
    t.finish()
}
