//! `menu::clear_record_entries` (0x47202F): the current record's event (`dword_7556D8`,
//! 0x618 an event, by the record's +0x3C) has its ten entries' (0x9C apart) +0x40 0.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `sub_4A3BE0`. `records::find_record`'s Rust version. Refuses a record past the
//! table or no events table, before anything. Self-test: the original and the Rust version
//! on the game's tables; put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.
use crate::errors::Refused;
use crate::framework::Regs;
use crate::menu::standings::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn clear_record_entries(entry: &Regs) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::menu::bests::clear_marks`.
    let event = current_event()?;
    rc2k_core::menu::bests::clear_marks(&mut crate::menu::rank_record_entry::event_list(event));
    Ok(*entry)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { clear_record_entries(r) }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
