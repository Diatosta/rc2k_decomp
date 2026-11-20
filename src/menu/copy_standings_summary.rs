//! `menu::copy_standings_summary`: 0x5A entries of standings table A copied, in order, into the
//! summary table (0x50 bytes each: the entry's name, nationality, car and time): with
//! `state.menus.class_mode` 2 the entries that have a group, else those without.
//!
//! Refuses null tables and a table A that runs out (no end entry), before anything.
use crate::errors::{BadInput, Refused};
use crate::menu::owner::Screens;
use crate::menu::standings::*;
use crate::saves::best_times::BestTables;

pub fn copy_standings_summary(screens: &Screens, tables: &BestTables) -> Result<(), Refused> {
    if !tables.summary.is_taken() {
        return Err(BadInput("no standings summary").into());
    }
    let grouped = screens.state.menus.class_mode.get() == 2;
    let Some(rows) = rc2k_core::saves::rankings::summary_rows(&walked_entries_a(tables)?, grouped)
    else {
        return Err(BadInput("standings table A runs out").into());
    };
    tables.summary.with(|t| {
        for (row, [name_id, nation, car, time]) in t.rows.iter_mut().zip(rows) {
            row.name_id = name_id;
            row.nation = nation;
            row.car = car;
            row.time = time;
        }
    })?;
    Ok(())
}
