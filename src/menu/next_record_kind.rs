//! `menu::next_record_kind`: the kind (first dword) of the record after the
//! current one (the record table, section `section`, index `index` + 1).
//!
//! Callers: `saves::load_savegame`, the event drivers.
//! Refuses a section or index past 63 or a section the table does not hold, before anything. Rust
//! callers run its `entry` (`routes.rs`).

use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::system::owner::System;

/// The kind.
pub fn next_record_kind(menus: &Menus, system: &System) -> Result<u32, Refused> {
    let screens = &menus.screens;
    let section: u32 = screens.event.section.get();
    let index: u32 = screens.event.index.get();
    // The scan may run on past the 66 records (to `championship_end` and the event tables):
    // the tables from the championship on, in the image's order, to their end.
    let tables = &system.records().stage_tables;
    let table = tables.bytes_on(crate::records::stages_store::Piece::Championship);
    let kind = rc2k_core::menu::record_kinds::next_record_kind(&table, section, index)?;
    Ok(kind)
}
