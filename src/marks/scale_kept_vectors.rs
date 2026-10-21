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
//!
//! Converted 2026-10-21: the hook's Rust version is
//! `scale_vectors_by::scale_triples_plain`, through `rc2k_core::marks::view::scaled` (plain
//! floats; the x87 version stays for the Rust callers in unconverted code). Shadow mode and
//! the self-test compare within `service::floats`' tolerances (the self-test at both test
//! precisions, exact at double).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::marks::leaf_test::test_leaf_floats;
use crate::marks::scale_vectors_by::{inputs_of, results_of, scale_triples, scale_triples_plain};
use crate::scene::matrix::float_places;
use crate::screens::harness::Left;
use crate::service::floats::*;
use crate::service::harness::at;
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
    unsafe { scale_triples_plain(&PAIRS) };
    Ok(Left::nothing())
}

/// What it writes.
pub fn regions() -> Vec<crate::service::harness::Region> {
    results_of(&PAIRS).into_iter().map(at).collect()
}

extern "C" fn dispatch(r: &mut Regs) {
    let places = float_places(&regions());
    let floats = Floats {
        memory: &places,
        regs: &[],
    };
    dispatch_state_floats(&HOOK, r, call_original, &regions(), &floats, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let (inputs, written) = (inputs_of(&PAIRS), results_of(&PAIRS));
    test_leaf_floats(
        &mut t,
        "marks::scale_kept_vectors",
        &inputs,
        &written,
        Regs::SENTINEL,
        &[],
        8,
        call_original,
        &rust,
    );
    t.finish()
}
