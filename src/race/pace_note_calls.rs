//! `race::pace_note_calls` (0x441CB4): one pace note's codes into a group's text: from the
//! place ebx (checkpoint x 8 + slot), each code (`race::pace_note_code`) up to a 0 or 'd'
//! (0x64): a corner ('c', 0x63) turned into its sharpness 1 .. 5 (+ 8 long, + 0x10 one
//! way: from note edx + 1's end and start, see `race/pace_notes.rs`); 'b' (0x62) moves the
//! text on without writing; '/', '<', '4' .. '8' over a ')' just written take its place;
//! the rest written as they are, at the text's edi.
//!
//! In:  ebx the place, edx the note before, edi the text's place
//! Out: ebx, esi (its face), eax (0 or 0x64): where it stopped; edi on; ebp a corner's
//!      flags (0, 8, 0x10, 0x18) when there was one; ecx and edx kept (pushed); the
//!      slot `dword_614D34` that place's; no flags read
//!
//! Callers: `race::init_pace_notes` (twice). It jumps back to its fourth byte: hooked by a
//! private copy (its call relocated).
//! Refuses a place or a table place outside their room (see `race/pace_notes.rs`).
//! Self-test: the original and the Rust version on test stages (`pace_notes::TestStage`):
//! places, notes and text places by lot; cases the Rust version refuses are not run.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::pace_notes::{Course, Machine};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

/// The Rust version on a course.
pub fn pace_note_calls(r: &Regs, course: Course) -> Result<Left, crate::errors::Refused> {
    // Converted: `rc2k_core::race::pace_notes::Notes::note_calls`.
    let mut m = Machine::load(r, course);
    m.note_calls()?;
    m.commit();
    Ok(m.left())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &|r| {
        pace_note_calls(r, crate::race::pace_notes::game_course())
    })
}
