//! The pace notes' work: the distance words, the notes' starts, ends, end codes and gaps, the
//! groups' checkpoints and texts, and the texts' codes. Names are readings unless taken from the
//! game's own.

use crate::memory::session::{PlainBytes, PlainDword};

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct PaceNotes {
    /// The distance's word by tens, the first 41 bytes used (`race::pace_notes`).
    pub distance_words: PlainBytes<0x1034>,
    /// How many pace notes there are.
    pub note_count: PlainDword,
    /// The pace note groups (`physics::queue_pace_note`).
    pub group_count: PlainDword,
    /// Each note's start.
    pub note_starts: [PlainDword; 1024],
    /// Each note's end.
    pub note_ends: [PlainDword; 1024],
    /// Each note's end code.
    pub note_end_codes: [PlainDword; 1024],
    /// Each note's gap.
    pub note_gaps: [PlainDword; 1024],
    /// Each group's checkpoint (`physics::queue_pace_note`).
    pub group_checkpoints: [PlainDword; 1024],
    /// Each group's place in `text` (`physics::queue_pace_note`).
    pub group_texts: [PlainDword; 1024],
    /// The groups' codes (`physics::queue_pace_note`).
    pub text: PlainBytes<0x4000>,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece, with the image's first values.
impl PaceNotes {
    pub const fn new() -> Self {
        use crate::memory::cells::ZeroCells;
        use crate::memory::session::{PlainBytes, with_runs};
        Self {
            distance_words: PlainBytes::new(with_runs::<4148>(&[(
                1,
                b"    !!\"\"\"#####$$$$$%%%%%&&&&&''''''''''((((((((((",
            )])),
            ..ZeroCells::ZERO
        }
    }
}
