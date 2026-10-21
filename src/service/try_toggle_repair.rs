//! `service::try_toggle_repair` (0x46BFB9): works out whether flipping the flag of the
//! highlighted menu item fits the service time, and says so in `dword_75AAA4`.
//!
//! In:  nothing
//! Out: every register preserved
//! It flips the flag, adds up the times of the chosen items into `dword_75AC8C` (so the total
//! is the one with the flip), and takes `budget - total`:
//!   at least 900:  status 3, and the flag is flipped back;
//!   1 .. 899:      status 1, and the flag is flipped back;
//!   0 or less:     status 2, and the flag is set to 0.
//! So the flag ends as it was, except in the last case; the callers act on the status. What
//! the three statuses mean to them is not worked out yet (step B2).
//! Refuses a highlighted item past the flag table and what `chosen_repairs_time` refuses.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::framework::{game_read, game_write};
use crate::screens::harness::Left;
use crate::service::harness::{
    FLAG_PATTERNS, RepairTest, check_state, dispatch_state, repair_regions, set_flags,
};
use crate::service::state::{
    MENU_ITEM_RVA, REPAIR_TOTAL_RVA, STATUS_RVA, TIME_BUDGET_RVA, flagged_item,
};
use crate::service::state::{flags, store_flags, time_of};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "service::try_toggle_repair",
    rva(0x46BFB9),
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

pub unsafe fn try_toggle_repair() -> Result<(), Refused> {
    // Converted: `rc2k_core::service::repairs::try_toggle`.
    let item = flagged_item()?;
    let mut t = flags();
    let budget = unsafe { game_read::<i32>(TIME_BUDGET_RVA) };
    let (total, status) =
        rc2k_core::service::repairs::try_toggle(&mut t, item as usize, budget, time_of)
            .map_err(|e| crate::errors::BadInput(e.0))?;
    store_flags(&t);
    unsafe {
        game_write(REPAIR_TOTAL_RVA, total);
        game_write(STATUS_RVA, status);
    }
    Ok(())
}

// ---- hook ----

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { try_toggle_repair() }?;
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
    // Only items with a repair time: flipping another one on makes the original read through
    // a null pointer.
    for (n, chosen) in FLAG_PATTERNS.iter().enumerate() {
        for item in [2u32, 7, 0xF, 0x12] {
            for budget in [
                0i32, 500, 899, 900, 1000, 1099, 1100, 1101, 1700, 2000, 5000, 100_000, -50,
            ] {
                check_state(
                    &mut t,
                    &format!("pattern={n} item={item:#x} budget={budget}"),
                    call_original,
                    Regs::SENTINEL,
                    &|| unsafe {
                        set_flags(chosen, 1);
                        game_write(MENU_ITEM_RVA, item);
                        game_write(TIME_BUDGET_RVA, budget);
                        game_write(REPAIR_TOTAL_RVA, 0x5EED_0001u32);
                        game_write(STATUS_RVA, 0x5EED_0002u32);
                    },
                    &repair_regions(),
                    &rust,
                );
            }
        }
    }
    test.end();
    t.finish()
}
