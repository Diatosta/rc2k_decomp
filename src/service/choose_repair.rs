//! `service::choose_repair` (0x46C138): sets the flag of the highlighted menu item to 1
//! (`dword_759B6E`).
//!
//! In:  nothing
//! Out: every register preserved
//! Refuses a highlighted item past the flag table. The name is a reading.

use crate::errors::BadInput;
use crate::errors::Refused;
use crate::framework::game_write;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{
    FLAG_PATTERNS, RepairTest, check_state, dispatch_state, repair_regions, set_flags,
};
use crate::service::state::{MENU_ITEM_RVA, flagged_item, set_repair_flag};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("service::choose_repair", rva(0x46C138), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

pub unsafe fn choose_repair() -> Result<(), BadInput> {
    let item = flagged_item()?;
    set_repair_flag(item, 1);
    Ok(())
}

// ---- hook ----

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { choose_repair() }?;
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
    for item in [0u32, 2, 9, 0x13] {
        for mark in [0u32, 1] {
            check_state(
                &mut t,
                &format!("item={item:#x} marks={mark}"),
                call_original,
                Regs::SENTINEL,
                &|| unsafe {
                    set_flags(FLAG_PATTERNS[1], mark);
                    game_write(MENU_ITEM_RVA, item);
                },
                &repair_regions(),
                &rust,
            );
        }
    }
    test.end();
    t.finish()
}
