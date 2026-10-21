//! `service::clear_repair_marks` (0x46BA15): sets every entry of the second flag table
//! (`dword_759BC2`) to 0, up to its -1 end mark.
//!
//! In:  nothing
//! Out: every register preserved
//! The first entry is cleared without being checked for the end mark, as in the original.
//! Refuses a table whose end mark is not where it belongs. What the marks mean has not been
//! worked out: the name is a reading.

use crate::errors::BadInput;
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{
    FLAG_PATTERNS, RepairTest, check_state, dispatch_state, repair_regions, set_flags,
};
use crate::service::state::{marks, store_marks};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "service::clear_repair_marks",
    rva(0x46BA15),
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

// ---- Rust version ----

pub unsafe fn clear_repair_marks() -> Result<(), BadInput> {
    // Converted: `rc2k_core::service::repairs::clear`.
    let mut t = marks();
    rc2k_core::service::repairs::clear(&mut t)?;
    store_marks(&t);
    Ok(())
}

// ---- hook ----

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { clear_repair_marks() }?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &repair_regions(), &rust);
}

// ---- self-test: known repair times, several sets of chosen items ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let test = RepairTest::begin();
    RepairTest::set_known_times();
    for mark in [0u32, 1, 7] {
        check_state(
            &mut t,
            &format!("marks={mark}"),
            call_original,
            Regs::SENTINEL,
            &|| set_flags(FLAG_PATTERNS[1], mark),
            &repair_regions(),
            &rust,
        );
    }
    test.end();
    t.finish()
}
