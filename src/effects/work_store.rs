//! The loose variables from the x87 status word at 0x71C814 to the cosine search's constants (up to 0x71CA60): the status of the last compare (`scene::vector3`, the marks), the pixel shake, the edge scratch, the marks' random seed, the sound pair, the cosine search's constants. Names are readings unless IDA's.
//! Most of the dwords after 0x71C8D6 lie at addresses that are not a multiple of 4 (`Loose32`).

use crate::race::panel_store::Loose32;
use crate::session::{Bytes, Dword};

#[repr(C)]
pub struct MarksWork {
    /// +0x0 (0x71C814): `g_fpuStatus`: the last compare's C0 .. C2 (`scene::vector3`, the marks' compares).
    pub fpu_status: Dword,
    /// +0x4 (0x71C818): not named or used (a gap of 0x58 bytes).
    pub unknown_4: Bytes<88>,
    /// +0x5c (0x71C870): `flt_71C870`: 1 / 65536, the pixel shake (`effects::pixels`).
    pub shake: Dword,
    /// +0x60 (0x71C874): not named or used (a gap of 0xc bytes).
    pub unknown_60: Bytes<12>,
    /// +0x6c (0x71C880): `dword_71C880`: the scratch of the edge pair (`effects::spans`), the rows' lengths (`effects::turns`) and the windscreen drop cell's x (`effects::windscreen_drops_frame`).
    pub scratch: Dword,
    /// +0x70 (0x71C884): not named or used (a gap of 0x48 bytes).
    pub unknown_70: Bytes<72>,
    /// +0xb8 (0x71C8CC): `g_marksSeed`: the marks' random seed (times 0x9D plus 0x47 each draw).
    pub seed: Dword,
    /// +0xbc (0x71C8D0): not named or used (a gap of 0x12 bytes).
    pub unknown_bc: Bytes<18>,
    /// +0xce (0x71C8E2): `dword_71C8E2`: the sound the frame plays (`effects::draw_frame`).
    pub sound_source: Loose32,
    /// +0xd2 (0x71C8E6): `dword_71C8E6`: set to 0 with it (`effects::draw_frame`).
    pub sound_zero: Loose32,
    /// +0xd6 (0x71C8EA): not named or used (a gap of 0x14c bytes).
    pub unknown_d6: Bytes<332>,
    /// +0x222 (0x71CA36): `flt_71CA36`: pi / 2, the cosine search's start (`scene::vector3`).
    pub cosine_start: Loose32,
    /// +0x226 (0x71CA3A): `flt_71CA3A`: pi / 4, its step.
    pub cosine_step: Loose32,
    /// +0x22a (0x71CA3E): `flt_71CA3E`: 0.5.
    pub cosine_half: Loose32,
    /// +0x22e (0x71CA42): not named or used (a gap of 0x1e bytes).
    pub unknown_22e: Bytes<30>,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.
unsafe impl crate::pointers::Cells for MarksWork {}

/// The piece: a static of its own since 2026-10-29 (it left the race block, phase 13:
/// `race::store::LEFT`), with the image's first values.
pub static MARKS_WORK: MarksWork =
    crate::pointers::cells_of_bytes(crate::race::store::first_values::<{ END - START }>(START));

/// IDA 0x71C814, the first byte of the piece.
pub const START: usize = 0x71C814;
/// IDA 0x71CA60, the end of the piece.
pub const END: usize = 0x71CA60;

const _: () = assert!(END - START == core::mem::size_of::<MarksWork>());

/// Where IDA 0x71C814 is now (the piece's static).
pub fn piece_addr() -> u32 {
    crate::pointers::game_address(std::ptr::from_ref(&MARKS_WORK))
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(&MARKS_WORK, START, END);
    }

    #[test]
    fn piece_has_its_home() {
        assert_eq!(
            piece_addr() % (core::mem::align_of::<MarksWork>()) as u32,
            0
        );
        assert_eq!(piece_addr() as usize, crate::moved::place(START));
        assert_eq!(
            crate::moved::place(END - 1),
            (piece_addr() + (END - START) as u32 - 1) as usize
        );

        assert_eq!(offset_of!(MarksWork, fpu_status), 0x71C814 - START);
        assert_eq!(offset_of!(MarksWork, shake), 0x71C870 - START);
        assert_eq!(offset_of!(MarksWork, scratch), 0x71C880 - START);
        assert_eq!(offset_of!(MarksWork, seed), 0x71C8CC - START);
        assert_eq!(offset_of!(MarksWork, sound_source), 0x71C8E2 - START);
        assert_eq!(offset_of!(MarksWork, cosine_start), 0x71CA36 - START);
        assert_eq!(offset_of!(MarksWork, cosine_half), 0x71CA3E - START);
    }
}
