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
    /// +0x5c (0x71C870): `flt_71C870`: 1 / 65536, the pixel shake (`marks::pixels`).
    pub shake: Dword,
    /// +0x60 (0x71C874): not named or used (a gap of 0xc bytes).
    pub unknown_60: Bytes<12>,
    /// +0x6c (0x71C880): `dword_71C880`: the scratch of the edge pair (`marks::spans`), the rows' lengths (`marks::turns`) and the hexagon cell's x (`marks::hexagons_frame`).
    pub scratch: Dword,
    /// +0x70 (0x71C884): not named or used (a gap of 0x48 bytes).
    pub unknown_70: Bytes<72>,
    /// +0xb8 (0x71C8CC): `g_marksSeed`: the marks' random seed (times 0x9D plus 0x47 each draw).
    pub seed: Dword,
    /// +0xbc (0x71C8D0): not named or used (a gap of 0x12 bytes).
    pub unknown_bc: Bytes<18>,
    /// +0xce (0x71C8E2): `dword_71C8E2`: the sound the frame plays (`marks::draw_frame`).
    pub sound_source: Loose32,
    /// +0xd2 (0x71C8E6): `dword_71C8E6`: set to 0 with it (`marks::draw_frame`).
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

/// The piece: the race block's bytes at IDA 0x71C814 seen as `MarksWork`. They stay there
/// while code reaches them by RVA from `race::store::asm_base()` and by IDA address
/// (`moved::place`): one copy for all three.
pub struct InRaceBlock;

impl core::ops::Deref for InRaceBlock {
    type Target = MarksWork;
    fn deref(&self) -> &MarksWork {
        // SAFETY: the race block lives as long as the program and is aligned as the test
        // checks; every field is an atomic or a cell, as the block's bytes were shared.
        unsafe { &*(crate::pointers::host_at(piece_addr()) as *const MarksWork) }
    }
}

pub static MARKS_WORK: InRaceBlock = InRaceBlock;

/// IDA 0x71C814, the first byte of the piece.
pub const START: usize = 0x71C814;
/// IDA 0x71CA60, the end of the piece.
pub const END: usize = 0x71CA60;

const _: () = assert!(END - START == core::mem::size_of::<MarksWork>());

/// Where IDA 0x71C814 is now (in the race block).
pub fn piece_addr() -> usize {
    crate::race::store::home(START)
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn piece_sits_on_the_race_block() {
        assert_eq!(piece_addr() % core::mem::align_of::<MarksWork>(), 0);
        assert_eq!(piece_addr(), crate::moved::place(START));
        assert_eq!(
            crate::moved::place(END - 1),
            piece_addr() + (END - START) - 1
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
