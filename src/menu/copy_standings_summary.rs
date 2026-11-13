//! `menu::copy_standings_summary`: 0x5A entries of standings table A copied, in order, into the
//! summary table (0x50 each: +0 / +4 / +8 / +0xC from the entry's +0xC / +8 / +0x10 / +0x14):
//! with `state.menus.class_mode` 2 the entries with a group (+0x1C not 0), else those without.
//!
//! Refuses null tables and a table A that runs out (no end entry), before anything.
use crate::errors::{BadInput, Refused};
use crate::menu::owner::Screens;
use crate::menu::standings::*;
use crate::saves::best_times::SUMMARY;

pub fn copy_standings_summary(screens: &Screens) -> Result<(), Refused> {
    if !SUMMARY.is_taken() {
        return Err(BadInput("no standings summary").into());
    }
    let grouped = screens.state.menus.class_mode.get() == 2;
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
