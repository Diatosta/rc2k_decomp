//! `marks::turn_kept_zy` (0x4C14D2): the three kept vectors turned: z and y by the first angle's sine and cosine (`marks/kept_turns.rs`).
//!
//! In:  nothing
//! Out: every register kept
//!
//! Caller: `marks::turn_kept_vectors`. Self-test: the inputs by lot, compared on what it writes; put back
//! afterwards.
//!
//! Converted 2026-10-21: the hook's Rust version is `kept_turns::turn_kept_zy_plain`,
//! through `rc2k_core::marks::view::KeptVectors::turn_zy` (plain floats; the x87 version
//! stays for the Rust callers in unconverted code). Shadow mode and the self-test compare
//! within `service::floats`' tolerances (the self-test at both test precisions, exact at
//! double).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::marks::kept_turns::{FIRST_RVA, turn_kept_zy_plain, vector_dwords};
use crate::marks::leaf_test::test_leaf_floats;
use crate::scene::matrix::float_places;
use crate::screens::harness::Left;
use crate::service::floats::*;
use crate::service::harness::{Region, at};
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
    unsafe { turn_kept_zy_plain() };
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

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut inputs = vector_dwords();
    inputs.extend(FIRST_RVA);
    test_leaf_floats(
        &mut t,
        "marks::turn_kept_zy",
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
