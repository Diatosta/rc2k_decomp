//! `menu::copy_standings_summary` (0x470466): 0x5A entries of standings table A copied, in
//! order, into the summary table (`dword_7556F4`, 0x50 each: +0 / +4 / +8 / +0xC from the
//! entry's +0xC / +8 / +0x10 / +0x14): with `dword_75AB40` 2 the entries with a group
//! (+0x1C not 0), else those without.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `sub_473126`. Refuses null tables and a table A that runs out (the original walks on
//! past its end), before anything.
use crate::errors::{BadInput, Refused};
use crate::menu::standings::*;
use crate::menu::state_store::MENU_STATE;
use crate::saves::best_times::SUMMARY;

/// The Rust version (see the header): the registers it leaves.
pub fn copy_standings_summary() -> Result<(), Refused> {
    if !SUMMARY.is_taken() {
        return Err(BadInput("no standings summary").into());
    }
    let grouped = MENU_STATE.menus.class_mode.get() == 2;
    // Converted: `rc2k_core::saves::rankings::summary_rows`.
    let Some(rows) = rc2k_core::saves::rankings::summary_rows(&walked_entries_a()?, grouped) else {
        return Err(BadInput("standings table A runs out").into());
    };
    SUMMARY.with(|t| {
        for (row, [name_id, nation, car, time]) in t.rows.iter_mut().zip(rows) {
            row.name_id = name_id;
            row.nation = nation;
            row.car = car;
            row.time = time;
        }
    })?;
    Ok(())
}
