//! `menu::rank_record_entry` (0x4A3D75): the current record's event's tenth record entry
//! moved up (`menu::rank_record_entry::rank_entries`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `menu::enter_record_entry`. Refuses a record past the table or no events table, before
//! anything.
use crate::memory::blocks::{At, Place};
use crate::menu::standings::*;

/// An event's ten record entries (0x9C each from the event's start, the tenth at +0x57C),
/// for the core.
pub fn with_event_list<R>(
    event: At,
    f: impl FnOnce(&mut rc2k_core::menu::standings::Entries) -> R,
) -> Result<R, crate::errors::BadInput> {
    with_table(Place::Owned(event), 0x9C, 10, f)
}

/// The current record's event's ten record entries (+0x14 the time): the tenth moved up
/// past each entry whose time is not below its own (signed), each of +0xC, +0x14, +4, +8,
/// +0x1C, +0x10, +0x18, +0x40, +0x4C .. +0x58 and the name's 0x1F bytes at +0x20 swapped
/// (the name through `g_keptPlayerName`).
pub fn rank_entries(event: At) -> Result<(), crate::errors::BadInput> {
    // Converted: `rc2k_core::menu::bests::rank_record`.
    with_event_list(event, |list| {
        with_kept_name(|kept| rc2k_core::menu::bests::rank_record(list, kept))
    })
}
