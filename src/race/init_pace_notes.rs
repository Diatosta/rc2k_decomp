//! `race::init_pace_notes`: makes the pace notes' tables from the checkpoints' faces (see
//! `race/pace_notes.rs`). `PaceNotes.group_count` and `PaceNotes.note_count` are set to 0; with
//! fewer than two checkpoints that is all. Otherwise it finds the start (the first checkpoint of
//! face kind 2, or the count, + 2), the notes (start / end / the end's code), each note's distance
//! to the next one's start, and the groups (the distance word, then each note's calls, ')' between
//! joined notes, a NUL after each group but the last) with their count.
//!
//! Refuses what `race/pace_notes.rs` refuses: places outside the tables or the course, and a search
//! that would run on past the count (no face of kind 2 before the last checkpoint but one). Nothing
//! is written then.

use crate::errors::Refused;
use crate::race::owner::Race;
use crate::race::pace_notes::{Course, GameStage, Machine};

/// On the given course.
fn init_pace_notes_on(race: &Race, course: Course) -> Result<(), Refused> {
    // Logic in `rc2k_core::race::pace_notes::Notes::init`.
    let stage = GameStage::new(race);
    let mut m = Machine::load(race, course, &stage)?;
    m.init()?;
    m.commit(race)?;
    Ok(())
}

/// On the game's course.
pub(super) fn init_pace_notes(race: &Race) -> Result<(), Refused> {
    init_pace_notes_on(race, crate::race::pace_notes::game_course(race))
}
