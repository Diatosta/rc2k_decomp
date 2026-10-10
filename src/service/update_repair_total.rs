//! `service::update_repair_total` (0x46BA2C): adds up the times of the chosen items into
//! `dword_75AC8C` and, if the time budget minus that total is at least 900, clears the second
//! flag table (`clear_repair_marks`).
//!
//! In:  nothing
//! Out: every register preserved
//! Refuses what `state::chosen_repairs_time` refuses.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::framework::{game_read, game_write};
use crate::screens::harness::Left;
use crate::service::clear_repair_marks::clear_repair_marks;
use crate::service::harness::{
    FLAG_PATTERNS, RepairTest, check_state, dispatch_state, repair_regions, set_flags,
};
use crate::service::state::{
    FREE_SERVICE_TIME, REPAIR_TOTAL_RVA, TIME_BUDGET_RVA, chosen_repairs_time,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "service::update_repair_total",
    rva(0x46BA2C),
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

pub unsafe fn update_repair_total() -> Result<(), Refused> {
    let total = chosen_repairs_time()?;
    unsafe {
        let budget: i32 = game_read(TIME_BUDGET_RVA);
        if budget.wrapping_sub(total) >= FREE_SERVICE_TIME {
            clear_repair_marks()?;
        }
        game_write(REPAIR_TOTAL_RVA, total);
    }
    Ok(())
}

// ---- hook ----

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { update_repair_total() }?;
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
    for (n, chosen) in FLAG_PATTERNS.iter().enumerate() {
        for budget in [
            0i32, 500, 899, 900, 1000, 1099, 1100, 1101, 1700, 2000, 5000, 100_000, -50,
        ] {
            check_state(
                &mut t,
                &format!("pattern={n} budget={budget}"),
                call_original,
                Regs::SENTINEL,
                &|| unsafe {
                    set_flags(chosen, 1);
                    game_write(TIME_BUDGET_RVA, budget);
                    game_write(REPAIR_TOTAL_RVA, 0x5EED_0001u32);
                },
                &repair_regions(),
                &rust,
            );
        }
    }
    test.end();
    t.finish()
}
