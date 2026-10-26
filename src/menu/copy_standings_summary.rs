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
use crate::memory::blocks::Place;
use crate::menu::champ_store::CHAMP_VIEW;
use crate::menu::standings::*;
use crate::menu::state_store::MENU_STATE;

/// The Rust version (see the header): the registers it leaves.
pub fn copy_standings_summary() -> Result<(), Refused> {
    let (Some(summary), Some(first)) = (CHAMP_VIEW.summary.get(), TABLE_A_RVA.get()) else {
        return Err(BadInput("no standings tables").into());
    };
    let grouped = MENU_STATE.menus.class_mode.get() == 2;
    let first = Place::Owned(first);
    let all = entries(first, A_ENTRY)?;
    // Converted: `rc2k_core::menu::standings::summary_rows`.
    let rows = with_table(first, A_ENTRY, all.len() as u32, |t| {
        rc2k_core::menu::standings::summary_rows(t, grouped)
    })?;
    let Some(rows) = rows else {
        return Err(BadInput("standings table A runs out").into());
    };
    for (n, row) in rows.into_iter().enumerate() {
        let s = summary.add(0x50 * n as u32);
        for (k, v) in row.into_iter().enumerate() {
            s.add(4 * k as u32).write(v, "the standings summary")?;
        }
    }
    Ok(())
}
