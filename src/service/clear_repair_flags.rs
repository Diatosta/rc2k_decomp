//! `service::clear_repair_flags` (0x470DC3): sets every entry of the flag table
//! (`dword_759B6E`) to 0, up to its -1 end mark: nothing is chosen any more.
//!
//! In:  nothing
//! Out: every register preserved
//! The first entry is cleared without being checked for the end mark, as in the original.
//! Refuses a table whose end mark is not where it belongs.

use crate::errors::BadInput;
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{
    FLAG_PATTERNS, RepairTest, check_state, dispatch_state, repair_regions, set_flags,
};
use crate::service::state::{ITEM_COUNT, repair_flag, set_repair_flag};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "service::clear_repair_flags",
    rva(0x470DC3),
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

pub unsafe fn clear_repair_flags() -> Result<(), BadInput> {
    // Find the end first, so that nothing is written if the table is not as expected.
    let Some(end) = (1..=ITEM_COUNT).find(|&item| repair_flag(item) == u32::MAX) else {
        return Err(BadInput("the repair flag table has lost its end mark"));
    };
    for item in 0..end {
        set_repair_flag(item, 0);
    }
    Ok(())
}

// ---- hook ----

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { clear_repair_flags() }?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &repair_regions(), &rust);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let test = RepairTest::begin();
    for (n, chosen) in FLAG_PATTERNS.iter().enumerate() {
        check_state(
            &mut t,
            &format!("pattern={n}"),
            call_original,
            Regs::SENTINEL,
            &|| set_flags(chosen, 1),
            &repair_regions(),
            &rust,
        );
    }
    test.end();
    t.finish()
}
