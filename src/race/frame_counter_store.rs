//! The `framecounter` setting, 0x5EB100 (`FrameCounter.show`): shows the frame rate when not 0
//! (`race::draw_frame_rate`; `settings::read_settings` names it by address, by design).
//! Names are readings unless IDA's.

use crate::cells::ZeroCells;
#[cfg(test)]
use crate::race::owner::race;
use crate::session::PlainDword;

#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct FrameCounter {
    /// +0x0 (0x5EB100): `dword_5EB100`: the `framecounter` setting (`race::draw_frame_rate`).
    pub show: PlainDword,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece, with the image's first values.
impl FrameCounter {
    pub const fn new() -> Self {
        ZeroCells::ZERO
    }
}

/// IDA 0x5EB100, the first byte of the piece.
pub const START: usize = 0x5EB100;
/// IDA 0x5EB104, the end of the piece.
pub const END: usize = 0x5EB104;

const _: () = assert!(END - START == core::mem::size_of::<FrameCounter>());

/// The piece's host address (the alignment checks).
#[cfg(test)]
pub fn piece_addr() -> usize {
    std::ptr::from_ref(race().frame.counter()).addr()
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::test_support::first_values::assert_block_starts_as_the_games(race().frame.counter());
    }

    #[test]
    fn fields_are_at_the_games_offsets() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(offset_of!(FrameCounter, show), 0x0);
        assert_eq!(offset_of!(FrameCounter, show), 0x0);
        assert_eq!(offset_of!(FrameCounter, show), 0x0);
    }
}
