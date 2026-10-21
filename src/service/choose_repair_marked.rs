//! `service::choose_repair_marked` (0x46C111): sets the flag of the highlighted menu item to 1 in both
//! flag tables (`dword_759B6E` and `dword_759BC2`).
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
use crate::service::state::{MENU_ITEM_RVA, flagged_item};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "service::choose_repair_marked",
    rva(0x46C111),
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

pub unsafe fn choose_repair_marked() -> Result<(), BadInput> {
    // Converted: `rc2k_core::service::repairs::choose`.
    use crate::service::state::{flags, marks, store_flags, store_marks};
    let item = flagged_item()?;
    let (mut f, mut m) = (flags(), marks());
    rc2k_core::service::repairs::choose(&mut f, &mut m, item as usize, true);
    store_flags(&f);
    store_marks(&m);
    Ok(())
}

// ---- hook ----

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { choose_repair_marked() }?;
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
