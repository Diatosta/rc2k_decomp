//! `menu::rank_record_entry` (0x4A3D75): the current record's event's tenth record entry
//! moved up (`menu::rank_record_entry::rank_entries`).
//!
//! Caller: `menu::enter_record_entry`. Refuses a record past the table or no events table, before
//! anything.

use crate::menu::owner::Play;
use crate::menu::standings::*;

/// The event's ten record entries (`standings::current_event`'s place): the tenth moved up
/// past each entry whose time is not below its own (signed), the record fields and the
/// name's 0x1F bytes exchanged (the name through `g_keptPlayerName`).
pub fn rank_entries(play: &Play, event: usize) -> Result<(), crate::errors::BadInput> {
    // Converted: `rc2k_core::saves::entries::rank_record`.
    with_event(event, |list| {
        with_kept_name(play, |kept| {
            rc2k_core::saves::entries::rank_record(list, kept);
        });
    })
}
