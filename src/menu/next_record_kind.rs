//! `menu::next_record_kind` (0x470D08): the kind (first dword) of the record after the
//! current one (`g_recordTable`, section `section`, index `index` + 1), kept in
//! `draw_work.figure`.
//!
//! Callers: `saves::load_savegame`, the event drivers.
//! Refuses a section or index past 63 or a section the table does not hold, before anything. Rust
//! callers run its `entry` (`routes.rs`).

use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::system::owner::System;

/// The kind (eax in the original), also kept in
/// `DrawWork.figure`.
pub fn next_record_kind(menus: &Menus, system: &System) -> Result<u32, Refused> {
    let screens = &menus.screens;
    let section: u32 = screens.event.section.get();
    let index: u32 = screens.event.index.get();
    // The scan may run on past the 66 records (to `championship_end` and the event tables):
    // the tables from the championship on, in the image's order, to their end.
    let tables = &system.records().stage_tables;
    let table = tables.bytes_on(crate::records::stages_store::Piece::Championship);
    // The logic is in `rc2k_core::menu::record_kinds::next_record_kind`.
    let kind = rc2k_core::menu::record_kinds::next_record_kind(&table, section, index)?;
    screens.draw_work.figure.set(kind);
    Ok(kind)
}
