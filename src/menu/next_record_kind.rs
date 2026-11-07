//! `menu::next_record_kind` (0x470D08): the kind (first dword) of the record after the
//! current one (`g_recordTable`, section `section`, index `index` + 1), kept in
//! `dword_7559EC`.
//!
//! Callers: `saves::load_savegame`, the event drivers. `records::find_record`'s Rust version.
//! Refuses a section or index past 63 or a section the table does not hold, before anything. Rust
//! callers run its `entry` (`routes.rs`).

use crate::errors::Refused;
use crate::menu::owner::menus;
use crate::system::owner::system;

/// The Rust version: the kind (eax in the original), also kept in
/// `dword_7559EC`.
pub fn next_record_kind() -> Result<u32, Refused> {
    let screens = &menus().screens;
    let section: u32 = screens.event.section.get();
    let index: u32 = screens.event.index.get();
    // The scan may run on past the 66 records (to `championship_end` and the event tables):
    // bounded by the whole store.
    let table = crate::pointers::GamePtr::to(&system().records.stage_tables)
        .offset(
            std::mem::offset_of!(crate::records::stages_store::StageTables, championship) as u32,
        )
        .bytes("the championship's stage table")?;
    // Converted: `rc2k_core::menu::record_kinds::next_record_kind`.
    let kind = rc2k_core::menu::record_kinds::next_record_kind(&table, section, index)?;
    screens.draw_work.figure.set(kind);
    Ok(kind)
}
