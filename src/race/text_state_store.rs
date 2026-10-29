//! The frame text's state in the race block, 0x5C6604 .. 0x5C6610: `dword_5C6604`, set to 2 by
//! `race::draw_frame_rate` (the text drawing mode, a reading), and the two dwords after it
//! (not read by the game's code).
//! Names are readings unless IDA's.

use crate::session::Dword;

#[repr(C)]
pub struct TextState {
    /// +0x0 (0x5C6604): `dword_5C6604`: set to 2 by `race::draw_frame_rate` before the frame-rate text (a reading).
    pub text_mode: Dword,
    /// +0x4 (0x5C6608): not named.
    pub unknown_4: [Dword; 2],
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.
unsafe impl crate::pointers::Cells for TextState {}

/// The piece: a static of its own since 2026-10-29 (it left the race block, phase 13:
/// `race::store::LEFT`), with the image's first values.
pub static TEXT_STATE: TextState =
    crate::pointers::cells_of_bytes(crate::race::store::first_values::<{ END - START }>(START));

/// IDA 0x5C6604, the first byte of the piece.
pub const START: usize = 0x5C6604;
/// IDA 0x5C6610, the end of the piece.
pub const END: usize = 0x5C6610;

const _: () = assert!(END - START == core::mem::size_of::<TextState>());

/// Where IDA 0x5C6604 is now (the piece's static).
pub fn piece_addr() -> u32 {
    crate::pointers::game_address(std::ptr::from_ref(&TEXT_STATE))
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(&TEXT_STATE, START, END);
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
        assert_eq!(offset_of!(TextState, text_mode), 0x5C6604 - START);
        assert_eq!(offset_of!(TextState, text_mode), 0x5C6604 - START);
        assert_eq!(offset_of!(TextState, text_mode), 0x5C6604 - START);
    }
}
