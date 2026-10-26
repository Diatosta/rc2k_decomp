//! `menu::rank_record_entry` (0x4A3D75): the current record's event's tenth record entry
//! moved up (`menu::rank_record_entry::rank_entries`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `menu::enter_record_entry`. Refuses a record past the table or no events table,
//! before anything. Self-test: the original and the Rust version on the game's event, its
//! entries' times by lot; put back.
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

/// An event's ten record entries (0x9C each from the event's start, the tenth at +0x57C),
/// for the core.
pub fn event_list<'a>(event: u32) -> rc2k_core::menu::standings::Entries<'a> {
    unsafe { table(event, 0x9C, 10) }
}

/// The current record's event's ten record entries (+0x14 the time): the tenth moved up
/// past each entry whose time is not below its own (signed), each of +0xC, +0x14, +4, +8,
/// +0x1C, +0x10, +0x18, +0x40, +0x4C .. +0x58 and the name's 0x1F bytes at +0x20 swapped
/// (the name through `g_keptPlayerName`).
pub fn rank_entries(event: u32) {
    // Converted: `rc2k_core::menu::bests::rank_record`.
    rc2k_core::menu::bests::rank_record(&mut event_list(event), kept_name());
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn rank_record_entry(entry: &Regs) -> Result<Regs, Refused> {
    rank_entries(current_event()?);
    Ok(*entry)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { rank_record_entry(r) }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
