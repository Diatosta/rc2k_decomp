//! `race::next_pace_slot` (0x441A4B): the pace notes' slot (`dword_614D34`) on by one;
//! at 8 or past (unsigned) back to 0 and ebp (the checkpoint) on by one.
//!
//! In:  ebp the checkpoint, `dword_614D34` the slot
//! Out: ebp; the rest kept; no flags read (the callers `jmp` or `cmp` next)
//!
//! Callers: `race::init_pace_notes` (four times). See `race/pace_notes.rs`.
//! Self-test: the original and the Rust version on the game's slot (put back): slots 0,
//! 6, 7, 8 and past.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{EBP, Regs, game_write};
use crate::race::pace_notes::{Machine, SLOT_RVA};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

/// The Rust version: the slot written, ebp left.
pub fn next_pace_slot(r: &Regs) -> Result<Left, crate::errors::Refused> {
    let mut m = Machine::load(r, crate::race::pace_notes::game_course());
    m.next_slot();
    unsafe { game_write(SLOT_RVA, m.slot) };
    Ok(Left::nothing().with(EBP, m.r.ebp))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &next_pace_slot)
}
