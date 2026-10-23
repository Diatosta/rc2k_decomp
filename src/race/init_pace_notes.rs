//! `race::init_pace_notes` (0x441A66): loadlevel_init's "init pacenotes": the pace notes'
//! tables made from the checkpoints' faces (see `race/pace_notes.rs`). `dword_615E14` and
//! `dword_615E10` 0; with fewer than two checkpoints nothing more. Else the start (the
//! first checkpoint of face kind 2, or the count, + 2); the notes (start / end / the end's
//! code); each note's distance to the next one's start; the groups (`race::
//! pace_distance_word`, `race::pace_note_calls` for each note, ')' between notes joined,
//! a NUL after each group but the last) and their count.
//!
//! In:  nothing
//! Out: fewer than two checkpoints: every register kept; else eax, ebx, esi as the last
//!      walk left them, ecx the groups less one, edx the notes' count plus one, edi the
//!      text's end, ebp the search's end or a corner's flags; no flags read (loadlevel
//!      goes on with its log)
//!
//! Caller: `race::loadlevel_init` (its step 25), which calls the Rust version directly.
//! Refuses what `race/pace_notes.rs` refuses: places outside the tables or the course,
//! and a search that would run on past the count (no face of kind 2 before the last
//! checkpoint but one). Nothing is written then.
//! Self-test: the original and the Rust version on test stages (`pace_notes::TestStage`):
//! counts 0 .. 48, codes and distances by lot; cases the Rust version refuses are not run.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::pace_notes::{Course, Machine};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

/// The Rust version on a course.
pub fn init_pace_notes_on(r: &Regs, course: Course) -> Result<Left, Refused> {
    // Converted: `rc2k_core::race::pace_notes::Notes::init`.
    let mut m = Machine::load(r, course);
    m.init()?;
    m.commit();
    Ok(m.left())
}

/// The Rust version on the game's course: the registers it leaves.
pub fn init_pace_notes(r: &mut Regs) -> Result<(), Refused> {
    init_pace_notes_on(r, crate::race::pace_notes::game_course())?.apply(r);
    Ok(())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &|r| {
        init_pace_notes_on(r, crate::race::pace_notes::game_course())
    })
}
