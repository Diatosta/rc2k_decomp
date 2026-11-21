//! `menu::rank_record_entry`: the current record's event's tenth record entry
//! moved up (`menu::rank_record_entry::rank_entries`).
//!
//! Refuses a record past the table or no events table, before anything.

use crate::menu::owner::Play;
use crate::menu::standings::*;

/// The event's ten record entries (`standings::current_event`'s place): the tenth moved up
/// past each entry whose time is not below its own (signed), the record fields and the
/// name's 0x1F bytes exchanged (the name through the kept player name).
pub(super) fn rank_entries(
    play: &Play,
    event: usize,
    tables: &crate::saves::best_times::BestTables,
) -> Result<(), crate::errors::BadInput> {
    with_event(tables, event, |list| {
        with_kept_name(play, |kept| {
            rc2k_core::saves::entries::rank_record(list, kept);
        });
    })
}
