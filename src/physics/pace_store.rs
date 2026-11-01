//! The pace notes' work (IDA 0x614D38 .. 0x61FE18; phase 7, typed): the distance words, the
//! notes' starts, ends, end codes and gaps, the groups' checkpoints and texts, and the texts'
//! codes. `#[repr(C)]` with the image's layout, over the race block's bytes (`PACE`). Names
//! are readings unless IDA's.

use crate::session::{Bytes, Dword};

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct PaceNotes {
    /// +0x0 (0X614D38): `g_paceDistanceWords`: the distance's word by tens, the first 41 bytes used (`race::pace_notes`); IDA names its first byte only.
    pub distance_words: Bytes<0x1034>,
    /// +0x1034 (0X615D6C): `dword_615D6C`: not worked out.
    pub unknown_d6c: [Dword; 41],
    /// +0x10d8 (0X615E10): `g_paceNoteCount`.
    pub note_count: Dword,
    /// +0x10dc (0X615E14): `g_paceGroupCount`: the pace note groups (`physics::queue_pace_note`).
    pub group_count: Dword,
    /// +0x10e0 (0X615E18): `g_paceNoteStarts`.
    pub note_starts: [Dword; 1024],
    /// +0x20e0 (0X616E18): `g_paceNoteEnds`.
    pub note_ends: [Dword; 1024],
    /// +0x30e0 (0X617E18): `g_paceNoteEndCodes`.
    pub note_end_codes: [Dword; 1024],
    /// +0x40e0 (0X618E18): `g_paceNoteGaps`.
    pub note_gaps: [Dword; 1024],
    /// +0x50e0 (0X619E18): `g_paceGroupCheckpoints` (`physics::queue_pace_note`).
    pub group_checkpoints: [Dword; 1024],
    /// +0x60e0 (0X61AE18): `g_paceGroupTexts`: each group's place in `text` (`physics::queue_pace_note`; IDA's `int[1023]` stops short of the last).
    pub group_texts: [Dword; 1024],
    /// +0x70e0 (0X61BE18): `g_paceText`: the groups' codes (`physics::queue_pace_note`).
    pub text: Bytes<0x4000>,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece: a static of its own since 2026-10-29 (it left the race block, phase 13:
/// `race::store::LEFT`), with the image's first values.
pub static PACE: PaceNotes =
    crate::pointers::cells_of_bytes(crate::race::store::first_values::<{ END - START }>(START));

/// IDA 0x614D38, the first byte of the piece.
pub const START: usize = 0x614D38;
/// IDA 0x61FE18, the end of the piece.
pub const END: usize = 0x61FE18;

const _: () = assert!(END - START == core::mem::size_of::<PaceNotes>());

/// Where IDA 0x614D38 is now.
pub fn piece_addr() -> u32 {
    crate::pointers::game_address(core::ptr::from_ref(&PACE))
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(&PACE, START, END);
    }

    #[test]
    fn piece_has_its_home() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(
            crate::moved::place(END - 1),
            (piece_addr() + (END - START) as u32 - 1) as usize
        );
        assert_eq!(offset_of!(PaceNotes, group_count), 0x615E14 - START);
        assert_eq!(offset_of!(PaceNotes, note_gaps), 0x618E18 - START);
        assert_eq!(offset_of!(PaceNotes, group_checkpoints), 0x619E18 - START);
        assert_eq!(offset_of!(PaceNotes, group_texts), 0x61AE18 - START);
        assert_eq!(offset_of!(PaceNotes, text), 0x61BE18 - START);
    }
}
