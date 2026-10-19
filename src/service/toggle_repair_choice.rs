//! `service::toggle_repair_choice` (0x46B7C0): the item action of the service area's repair
//! items: the status (`dword_75AAA4`) and edit mode 0, the highlighted item's flag flipped,
//! the chosen repairs' time added up into `dword_75AC8C`, and by `budget - total` (signed):
//!   at least 900:  every mark (`dword_759BC2`, from entry 0 to the end mark) 0;
//!   0 .. 899:      the item's mark 1;
//!   below 0:       the flag flipped back and the status 1.
//! Then the total is added up and stored again.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//! Refuses (before anything) a highlighted item past the flag table and what
//! `chosen_repairs_time` refuses (an item without a time chosen: the original reads
//! through null), and a mark table without its end. Self-test: known repair times, the
//! flag patterns of `try_toggle_repair`'s test, items with a time, budgets around both
//! boundaries.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{
    FLAG_PATTERNS, Region, RepairTest, at, capture, check_state, dispatch_state, repair_regions,
    restore, set_flags,
};
use crate::service::state::{
    EDIT_MODE_RVA, FREE_SERVICE_TIME, MENU_ITEM_RVA, REPAIR_TOTAL_RVA, STATUS_RVA, TIME_BUDGET_RVA,
    chosen_repairs_time, flagged_item, marks_ended, repair_flag, repair_mark_addr, set_repair_flag,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "service::toggle_repair_choice",
    rva(0x46B7C0),
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

fn regions() -> Vec<Region> {
    let mut r = repair_regions().to_vec();
    r.push(at(EDIT_MODE_RVA));
    r
}

/// The Rust version (see the header).
pub unsafe fn toggle_repair_choice() -> Result<(), Refused> {
    let item = flagged_item()?;
    if !marks_ended() {
        return Err(BadInput("the repair mark table has lost its end mark").into());
    }
    let saved = capture(&regions());
    let result = unsafe { toggle(item) };
    if result.is_err() {
        restore(&regions(), &saved);
    }
    result
}

unsafe fn toggle(item: u32) -> Result<(), Refused> {
    unsafe {
        game_write(STATUS_RVA, 0u32);
        game_write(EDIT_MODE_RVA, 0u32);
        set_repair_flag(item, repair_flag(item) ^ 1);
        let total = chosen_repairs_time()?;
        game_write(REPAIR_TOTAL_RVA, total);
        let left = game_read::<i32>(TIME_BUDGET_RVA).wrapping_sub(total);
        if left >= FREE_SERVICE_TIME {
            // From entry 0, each written before the next is looked at for the end mark.
            let mut n = 0;
            loop {
                (repair_mark_addr(n) as *mut u32).write_unaligned(0);
                n += 1;
                if (repair_mark_addr(n) as *const u32).read_unaligned() == u32::MAX {
                    break;
                }
            }
        } else if left >= 0 {
            (repair_mark_addr(item) as *mut u32).write_unaligned(1);
        } else {
            set_repair_flag(item, repair_flag(item) ^ 1);
            game_write(STATUS_RVA, 1u32);
        }
        let total = chosen_repairs_time()?;
        game_write(REPAIR_TOTAL_RVA, total);
    }
    Ok(())
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { toggle_repair_choice() }?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let test = RepairTest::begin();
    let mode = [at(EDIT_MODE_RVA)];
    let saved_mode = capture(&mode);
    RepairTest::set_known_times();
    for (n, chosen) in FLAG_PATTERNS.iter().enumerate() {
        for item in [2u32, 7, 0xF, 0x12] {
            for budget in [
                -50i32, 0, 500, 899, 900, 1000, 1099, 1100, 1101, 1999, 2000, 100_000,
            ] {
                check_state(
                    &mut t,
                    &format!("pattern={n} item={item:#x} budget={budget}"),
                    call_original,
                    Regs::SENTINEL,
                    &|| unsafe {
                        set_flags(chosen, 0x5EED);
                        game_write(MENU_ITEM_RVA, item);
                        game_write(TIME_BUDGET_RVA, budget);
                        game_write(REPAIR_TOTAL_RVA, 0x5EED_0001u32);
                        game_write(STATUS_RVA, 0x5EED_0002u32);
                        game_write(EDIT_MODE_RVA, 1u32);
                    },
                    &regions(),
                    &rust,
                );
            }
        }
    }
    restore(&mode, &saved_mode);
    test.end();
    t.finish()
}
