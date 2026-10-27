//! `menu::clear_record_entries` (0x47202F): the current record's event (`dword_7556D8`,
//! 0x618 an event, by the record's +0x3C) has its ten entries' (0x9C apart) +0x40 0.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `sub_4A3BE0`. `records::find_record`'s Rust version. Refuses a record past the table or
//! no events table, before anything.
use crate::errors::Refused;
use crate::menu::standings::*;

/// The Rust version (see the header).
pub fn clear_record_entries() -> Result<(), Refused> {
    // Converted: `rc2k_core::saves::entries::clear_marks`.
    let event = current_event()?;
    with_event(event, rc2k_core::saves::entries::clear_marks)?;
    Ok(())
}
