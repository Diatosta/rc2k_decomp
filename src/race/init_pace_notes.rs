//! `race::init_pace_notes` (0x441A66): loadlevel_init's "init pacenotes": the pace notes' tables
//! made from the checkpoints' faces (see `race/pace_notes.rs`). `PaceNotes.group_count` and `PaceNotes.note_count`
//! 0; with fewer than two checkpoints nothing more. Else the start (the first checkpoint of face
//! kind 2, or the count, + 2); the notes (start / end / the end's code); each note's distance to
//! the next one's start; the groups (`race:: pace_distance_word`, `race::pace_note_calls` for each
//! note, ')' between notes joined, a NUL after each group but the last) and their count.
//!
//! Caller: `race::loadlevel_init` (its step 25), which calls it directly. Refuses
//! what `race/pace_notes.rs` refuses: places outside the tables or the course, and a search
//! that would run on past the count (no face of kind 2 before the last checkpoint but one).
//! Nothing is written then.

use crate::errors::Refused;
use crate::race::owner::Race;
use crate::race::pace_notes::{Course, Machine};

/// On a course.
pub fn init_pace_notes_on(race: &Race, course: Course) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::pace_notes::Notes::init`.
    let mut m = Machine::load(race, course)?;
    m.init()?;
    m.commit(race)?;
    Ok(())
}

/// On the game's course.
pub fn init_pace_notes(race: &Race) -> Result<(), Refused> {
    init_pace_notes_on(race, crate::race::pace_notes::game_course(&race.shared))
}
