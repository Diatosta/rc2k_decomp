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
use crate::menu::owner::Menus;
use crate::service::state::{flags, marks, store_marks, time_of};

// ---- Rust version ----

pub fn update_repair_total(menus: &Menus) -> Result<(), Refused> {
    let (screens, tables) = (&menus.screens, &menus.tables);
    // Converted: `rc2k_core::service::repairs::update_total`.
    let mut m = marks(tables);
    let budget = screens.event.service_minutes.get() as i32;
    let total = rc2k_core::service::repairs::update_total(&flags(tables), &mut m, budget, |item| {
        time_of(menus, tables, item)
    })
    .map_err(|e| crate::errors::BadInput(e.0))?;
    store_marks(tables, &m);
    screens.state.driver_stats.repair_time.set(total as u32);
    Ok(())
}
