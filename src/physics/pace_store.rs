//! The pace notes' work (IDA 0x614D38 .. 0x61FE18; phase 7, typed): the distance words, the
//! notes' starts, ends, end codes and gaps, the groups' checkpoints and texts, and the texts'
//! codes. `#[repr(C)]` with the image's layout, over the race block's bytes (`PACE`). Names
//! are readings unless IDA's.

#[cfg(test)]
use crate::race::owner::race;
use crate::session::{PlainBytes, PlainDword};

#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct PaceNotes {
    /// +0x0 (0X614D38): `g_paceDistanceWords`: the distance's word by tens, the first 41 bytes used (`race::pace_notes`); IDA names its first byte only.
    pub distance_words: PlainBytes<0x1034>,
    /// +0x1034 (0X615D6C): `dword_615D6C`: a table indexed by a distance word, read once at 0x441CA8 (`race::pace_distance_word`) with its result thrown away; no other use.
    pub discarded_distance_table: [PlainDword; 41],
    /// +0x10d8 (0X615E10): `g_paceNoteCount`.
    pub note_count: PlainDword,
    /// +0x10dc (0X615E14): `g_paceGroupCount`: the pace note groups (`physics::queue_pace_note`).
    pub group_count: PlainDword,
    /// +0x10e0 (0X615E18): `g_paceNoteStarts`.
    pub note_starts: [PlainDword; 1024],
    /// +0x20e0 (0X616E18): `g_paceNoteEnds`.
    pub note_ends: [PlainDword; 1024],
    /// +0x30e0 (0X617E18): `g_paceNoteEndCodes`.
    pub note_end_codes: [PlainDword; 1024],
    /// +0x40e0 (0X618E18): `g_paceNoteGaps`.
    pub note_gaps: [PlainDword; 1024],
    /// +0x50e0 (0X619E18): `g_paceGroupCheckpoints` (`physics::queue_pace_note`).
    pub group_checkpoints: [PlainDword; 1024],
    /// +0x60e0 (0X61AE18): `g_paceGroupTexts`: each group's place in `text` (`physics::queue_pace_note`; IDA's `int[1023]` stops short of the last).
    pub group_texts: [PlainDword; 1024],
    /// +0x70e0 (0X61BE18): `g_paceText`: the groups' codes (`physics::queue_pace_note`).
    pub text: PlainBytes<0x4000>,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece, with the image's first values.
impl PaceNotes {
    pub const fn new() -> Self {
        use crate::cells::ZeroCells;
        use crate::session::{PlainBytes, PlainDword, with_runs};
        Self {
            distance_words: PlainBytes::new(with_runs::<4148>(&[(
                1,
                b"    !!\"\"\"#####$$$$$%%%%%&&&&&''''''''''((((((((((",
            )])),
            discarded_distance_table: {
                let mut v: [PlainDword; 41] = ZeroCells::ZERO;
                v[32] = PlainDword::new(30);
                v[33] = PlainDword::new(50);
                v[34] = PlainDword::new(70);
                v[35] = PlainDword::new(100);
                v[36] = PlainDword::new(150);
                v[37] = PlainDword::new(200);
                v[38] = PlainDword::new(250);
                v[39] = PlainDword::new(0x12C);
                v[40] = PlainDword::new(0x190);
                v
            },
            ..ZeroCells::ZERO
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_values_are_the_games() {
        crate::test_support::first_values::assert_block_starts_as_the_games(race().physics.pace());
    }
}
