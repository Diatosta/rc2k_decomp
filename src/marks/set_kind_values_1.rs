//! `marks::set_kind_values_1` (0x4C7322): `init_marks`' step 8, the marks' kind's
//! values of set 1.
//!
//! In:  nothing
//! Out: eax, ebx, ecx the kind's copied three, ebp the kind times 12
//!
//! See `marks/kind_values.rs`. One caller, `marks::init_marks`. Refuses a kind past the
//! tables (0x22). Self-test: the original and the Rust version on the game's tables
//! filled with values of the test's, kinds 0 .. 0x21; the refusal; put back afterwards.
//!
//! Converted 2026-10-21: the hook's Rust version is `set_kind_values_plain`, through
//! `rc2k_core::marks::kind_values::values` (plain floats; the x87 version stays for
//! `marks::init_marks`). Shadow mode and the self-test compare within `service::floats`'
//! tolerances (the self-test at both test precisions, exact at double).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::marks::kind_values::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::floats::{Floats, dispatch_state_floats};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::set_kind_values_1", rva(0x4C7322), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { set_kind_values_plain(r, &FIRST) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    let places = kind_floats(&FIRST);
    let floats = Floats {
        memory: &places,
        regs: &[],
    };
    dispatch_state_floats(
        &HOOK,
        r,
        call_original,
        &kind_regions(&FIRST),
        &floats,
        &rust,
    );
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    crate::marks::kind_values::test_kinds(
        &mut t,
        "marks::set_kind_values_1",
        &FIRST,
        call_original,
        &rust,
    );
    t.finish()
}
