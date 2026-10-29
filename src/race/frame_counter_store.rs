//! The `framecounter` setting, 0x5EB100 (`dword_5EB100`): shows the frame rate when not 0
//! (`race::draw_frame_rate`; `settings::read_settings` names it by address, by design).
//! Names are readings unless IDA's.

use crate::session::Dword;

#[repr(C)]
pub struct FrameCounter {
    /// +0x0 (0x5EB100): `dword_5EB100`: the `framecounter` setting (`race::draw_frame_rate`).
    pub show: Dword,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.
unsafe impl crate::pointers::Cells for FrameCounter {}

/// The piece: a static of its own since 2026-10-29 (it left the race block, phase 13:
/// `race::store::LEFT`), with the image's first values.
pub static FRAME_COUNTER: FrameCounter =
    crate::pointers::cells_of_bytes(crate::race::store::first_values::<{ END - START }>(START));

/// IDA 0x5EB100, the first byte of the piece.
pub const START: usize = 0x5EB100;
/// IDA 0x5EB104, the end of the piece.
pub const END: usize = 0x5EB104;

const _: () = assert!(END - START == core::mem::size_of::<FrameCounter>());

/// Where IDA 0x5EB100 is now (the piece's static).
pub fn piece_addr() -> u32 {
    crate::pointers::game_address(std::ptr::from_ref(&FRAME_COUNTER))
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(&FRAME_COUNTER, START, END);
    }

    #[test]
    fn piece_has_its_home() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(
            piece_addr(),
            crate::framework::game_addr(crate::framework::rva(START))
        );
        assert_eq!(
            crate::moved::place(END - 1),
            (piece_addr() + (END - START) as u32 - 1) as usize
        );
        assert_eq!(offset_of!(FrameCounter, show), 0x5EB100 - START);
        assert_eq!(offset_of!(FrameCounter, show), 0x5EB100 - START);
        assert_eq!(offset_of!(FrameCounter, show), 0x5EB100 - START);
    }
}
