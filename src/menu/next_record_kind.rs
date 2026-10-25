//! `menu::next_record_kind` (0x470D08): the kind (first dword) of the record after the
//! current one (`g_recordTable`, section `section`, index `index` + 1), kept in
//! `dword_7559EC`.
//!
//! In:  nothing
//! Out: eax the kind; the rest kept
//!
//! Callers: `saves::load_savegame`, the event drivers. `records::find_record`'s Rust version.
//! Refuses a section or index past 63 or a section the table does not hold, before anything. Rust
//! callers run its `entry` (`routes.rs`); the registers above are the original's, for reference.

use crate::bridge::Raw;
use crate::errors::Refused;
use crate::menu::draw_work_store::DRAW_WORK;
use crate::menu::event_store::EVENT_STATE;

/// The Rust version (see the header): the kind (eax in the original), also kept in
/// `dword_7559EC`.
pub unsafe fn next_record_kind() -> Result<u32, Refused> {
    let section: u32 = { EVENT_STATE.section.get() };
    let index: u32 = { EVENT_STATE.index.get() };
    let table =
        unsafe { Raw::new(crate::records::stages_store::STAGE_TABLES.championship() as *const u8) };
    // Converted: `rc2k_core::menu::record_kinds::next_record_kind`.
    let kind = rc2k_core::menu::record_kinds::next_record_kind(&table, section, index)?;
    DRAW_WORK.figure.set(kind);
    Ok(kind)
}
