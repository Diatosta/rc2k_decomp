//! `service::update_repair_total` (0x46BA2C): adds up the times of the chosen items into
//! `dword_75AC8C` and, if the time budget minus that total is at least 900, clears the second
//! flag table (`clear_repair_marks`).
//!
//! In:  nothing
//! Out: every register preserved
//! Refuses what `state::chosen_repairs_time` refuses.
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::{game_read, game_write};
use crate::service::state::{REPAIR_TOTAL_RVA, TIME_BUDGET_RVA};
use crate::service::state::{flags, marks, store_marks, time_of};

// ---- Rust version ----

pub unsafe fn update_repair_total() -> Result<(), Refused> {
    // Converted: `rc2k_core::service::repairs::update_total`.
    let mut m = marks();
    let budget = unsafe { game_read::<i32>(TIME_BUDGET_RVA) };
    let total = rc2k_core::service::repairs::update_total(&flags(), &mut m, budget, time_of)
        .map_err(|e| crate::errors::BadInput(e.0))?;
    store_marks(&m);
    unsafe { game_write(REPAIR_TOTAL_RVA, total) };
    Ok(())
}
