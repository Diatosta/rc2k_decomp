//! The loose variables from the x87 status word at 0x71C814 to the cosine search's constants (up to 0x71CA60): the status of the last compare (`scene::vector3`, the marks), the pixel shake, the edge scratch, the marks' random seed, the sound pair, the cosine search's constants. Names are readings unless IDA's.
//! Most of the dwords after 0x71C8D6 lie at addresses that are not a multiple of 4 (`Loose32`).

#[cfg(test)]
use crate::effects::owner::effects;
use crate::race::panel_store::Loose32;
use crate::session::{PlainBytes, PlainDword, PlainFloat};

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct MarksWork {
    /// +0x0 (0x71C814): `g_fpuStatus`: the last compare's C0 .. C2 (`scene::vector3`, the marks' compares).
    pub fpu_status: PlainDword,
    /// +0x4 (0x71C818): an unreferenced gap (0x58 bytes).
    pub unused_4: PlainBytes<88>,
    /// +0x5c (0x71C870): `flt_71C870`: 1 / 65536, the pixel shake (`effects::pixels`).
    pub shake: PlainFloat,
    /// +0x60 (0x71C874): an unreferenced gap (0xc bytes).
    pub unused_60: PlainBytes<12>,
    /// +0x6c (0x71C880): `dword_71C880`: the scratch of the edge pair (`effects::spans`), the rows' lengths (`effects::turns`) and the windscreen drop cell's x (`effects::windscreen_drops_frame`).
    pub scratch: PlainDword,
    /// +0x70 (0x71C884): an unreferenced gap (0x48 bytes).
    pub unused_70: PlainBytes<72>,
    /// +0xb8 (0x71C8CC): `g_marksSeed`: the marks' random seed (times 0x9D plus 0x47 each draw).
    pub seed: PlainDword,
    /// +0xbc (0x71C8D0): an unreferenced gap (0x12 bytes).
    pub unused_bc: PlainBytes<18>,
    /// +0xce (0x71C8E2): `dword_71C8E2`: the sound the frame plays (`effects::draw_frame`).
    pub sound_source: Loose32,
    /// +0xd2 (0x71C8E6): `dword_71C8E6`: set to 0 with it (`effects::draw_frame`).
    pub sound_zero: Loose32,
    /// +0xd6 (0x71C8EA): an unreferenced gap (0x14c bytes).
    pub unused_d6: PlainBytes<332>,
    /// +0x222 (0x71CA36): `flt_71CA36`: pi / 2, the cosine search's start (`scene::vector3`).
    pub cosine_start: Loose32,
    /// +0x226 (0x71CA3A): `flt_71CA3A`: pi / 4, its step.
    pub cosine_step: Loose32,
    /// +0x22a (0x71CA3E): `flt_71CA3E`: 0.5.
    pub cosine_half: Loose32,
    /// +0x22e (0x71CA42): an unreferenced gap (0x1e bytes).
    pub unused_22e: PlainBytes<30>,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

impl MarksWork {
    /// The piece, with the image's first values.
    pub const fn new() -> Self {
        crate::cells::cells_of_bytes(crate::race::store::first_values::<{ END - START }>(START))
    }
}

/// IDA 0x71C814, the first byte of the piece.
pub const START: usize = 0x71C814;
/// IDA 0x71CA60, the end of the piece.
pub const END: usize = 0x71CA60;

const _: () = assert!(END - START == core::mem::size_of::<MarksWork>());

/// The piece's host address (the alignment checks).
#[cfg(test)]
pub fn piece_addr() -> usize {
    std::ptr::from_ref(&effects().setup.work).addr()
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::test_support::first_values::assert_block_starts_as_the_games(&effects().setup.work);
    }

    #[test]
    fn piece_has_its_home() {
        assert_eq!(piece_addr() % core::mem::align_of::<MarksWork>(), 0);

        assert_eq!(offset_of!(MarksWork, fpu_status), 0x0);
        assert_eq!(offset_of!(MarksWork, shake), 0x5c);
        assert_eq!(offset_of!(MarksWork, scratch), 0x6c);
        assert_eq!(offset_of!(MarksWork, seed), 0xb8);
        assert_eq!(offset_of!(MarksWork, sound_source), 0xce);
        assert_eq!(offset_of!(MarksWork, cosine_start), 0x222);
        assert_eq!(offset_of!(MarksWork, cosine_half), 0x22a);
    }
}
