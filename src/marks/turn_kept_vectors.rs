//! `marks::turn_kept_vectors` (0x4C13CA): the three kept vectors turned: the sines and cosines of `flt_76923E` and `flt_76924A`, then
//! `marks::turn_kept_xz` and `marks::turn_kept_zy` (`marks/kept_turns.rs`).
//!
//! In:  nothing
//! Out: every register kept
//!
//! Caller: `sub_4C1270`. Self-test: the inputs by lot, compared on what it writes; put back
//! afterwards.
//!
//! Converted 2026-10-21: the hook's Rust version is `kept_turns::turn_kept_vectors_plain`,
//! through `rc2k_core::marks::view::turn_kept_vectors` (plain floats; the x87 version stays
//! for the Rust callers in unconverted code). Shadow mode and the self-test compare within
//! `service::floats`' tolerances (the self-test at both test precisions, exact at double).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::marks::kept_turns::{
    ANGLES_RVA, FIRST_RVA, SECOND_RVA, turn_kept_vectors_plain, vector_dwords,
};
use crate::marks::leaf_test::test_leaf_floats;
use crate::scene::matrix::float_places;
use crate::screens::harness::Left;
use crate::service::floats::*;
use crate::service::harness::{Region, at};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::turn_kept_vectors", rva(0x4C13CA), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn written() -> Vec<usize> {
    let mut written = vector_dwords();
    written.extend(FIRST_RVA);
    written.extend(SECOND_RVA);
    written
}

/// What it writes.
pub fn regions() -> Vec<Region> {
    written().into_iter().map(at).collect()
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { turn_kept_vectors_plain() };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions: Vec<Region> = written().into_iter().map(at).collect();
    let places = float_places(&regions);
    let floats = Floats {
        memory: &places,
        regs: &[],
    };
    dispatch_state_floats(&HOOK, r, call_original, &regions, &floats, &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut inputs = vector_dwords();
    inputs.extend(ANGLES_RVA);
    test_leaf_floats(
        &mut t,
        "marks::turn_kept_vectors",
        &inputs,
        &written(),
        Regs::SENTINEL,
        &[],
        10,
        call_original,
        &rust,
    );
    t.finish()
}
