//! The pace notes' work: the distance words, the notes' starts, ends, end codes and gaps, the
//! groups' checkpoints and texts, and the texts' codes. Plain fields, held by the race's
//! physics owner on the game's thread only (`race::owner_physics::Physics::pace`). Names are
//! readings unless taken from the game's own.

use crate::memory::session::with_runs;

pub struct PaceNotes {
    /// The distance's word by tens, the first 41 bytes used (`race::pace_notes`).
    pub distance_words: [u8; 0x1034],
    /// How many pace notes there are.
    pub note_count: u32,
    /// The pace note groups (`physics::queue_pace_note`).
    pub group_count: u32,
    /// Each note's start.
    pub note_starts: [u32; 1024],
    /// Each note's end.
    pub note_ends: [u32; 1024],
    /// Each note's end code.
    pub note_end_codes: [u32; 1024],
    /// Each note's gap.
    pub note_gaps: [u32; 1024],
    /// Each group's checkpoint (`physics::queue_pace_note`).
    pub group_checkpoints: [u32; 1024],
    /// Each group's place in `text` (`physics::queue_pace_note`).
    pub group_texts: [u32; 1024],
    /// The groups' codes (`physics::queue_pace_note`).
    pub text: [u8; 0x4000],
}

/// The piece, with the first values.
impl PaceNotes {
    pub const fn new() -> Self {
        Self {
            distance_words: with_runs::<0x1034>(&[(
                1,
                b"    !!\"\"\"#####$$$$$%%%%%&&&&&''''''''''((((((((((",
            )]),
            note_count: 0,
            group_count: 0,
            note_starts: [0; 1024],
            note_ends: [0; 1024],
            note_end_codes: [0; 1024],
            note_gaps: [0; 1024],
            group_checkpoints: [0; 1024],
            group_texts: [0; 1024],
            text: [0; 0x4000],
        }
    }
}
