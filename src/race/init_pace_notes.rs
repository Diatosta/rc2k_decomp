//! `race::init_pace_notes` (0x441A66): loadlevel_init's "init pacenotes": the pace notes'
//! tables made from the checkpoints' faces (see `race/pace_notes.rs`). `dword_615E14` and
//! `dword_615E10` 0; with fewer than two checkpoints nothing more. Else the start (the
//! first checkpoint of face kind 2, or the count, + 2); the notes (start / end / the end's
//! code); each note's distance to the next one's start; the groups (`race::
//! pace_distance_word`, `race::pace_note_calls` for each note, ')' between notes joined,
//! a NUL after each group but the last) and their count.
//!
//! In:  nothing
//! Out: every register kept (liveness)
//!
//! Caller: `race::loadlevel_init` (its step 25), which calls the Rust version directly. Refuses
//! what `race/pace_notes.rs` refuses: places outside the tables or the course, and a search
//! that would run on past the count (no face of kind 2 before the last checkpoint but one).
//! Nothing is written then.

use crate::errors::Refused;
use crate::race::owner_shared::Shared;
use crate::race::pace_notes::{Course, Machine};

/// The Rust version on a course. The original reads no register at entry and keeps them
/// all (liveness): the walk's registers start from nothing.
pub fn init_pace_notes_on(course: Course) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::pace_notes::Notes::init`.
    let mut m = Machine::load(course)?;
    m.init()?;
    m.commit()?;
    Ok(())
}

/// The Rust version on the game's course.
pub fn init_pace_notes(shared: &Shared) -> Result<(), Refused> {
    init_pace_notes_on(crate::race::pace_notes::game_course(shared))
}
