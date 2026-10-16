//! `marks::set_kind_values_2` (0x4C7FA4): `init_marks`' step 9, the marks' kind's
//! values of set 2.
//!
//! In:  nothing
//! Out: eax, ebx, ecx the kind's copied three, ebp the kind times 12
//!
//! See `marks/kind_values.rs`. One caller, `marks::init_marks`. Refuses a kind past the
//! tables (0x22). Self-test: the original and the Rust version on the game's tables
//! filled with values of the test's, kinds 0 .. 0x21; the refusal; put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::marks::kind_values::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::dispatch_state;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::set_kind_values_2", rva(0x4C7FA4), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { set_kind_values(r, &SECOND) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &kind_regions(&SECOND), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    crate::marks::kind_values::test_kinds(&mut t, &SECOND, call_original, &rust);
    t.finish()
}
