//! The frame text's state in the race block, 0x5C6604 .. 0x5C6610: `dword_5C6604`, set to 2 by
//! `race::draw_frame_rate` (the text drawing mode, a reading: also the debug font's colour,
//! `debug::draw`), and the two dwords after it (the debug font's pixel, kept per depth).
//! Names are readings unless IDA's.

#[cfg(test)]
use crate::race::owner::race;
use crate::session::Dword;

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct TextState {
    /// +0x0 (0x5C6604): `dword_5C6604`: set to 2 by `race::draw_frame_rate` before the frame-rate text; the debug font's colour number (`debug::draw::COLOURS`).
    pub text_mode: Dword,
    /// +0x4 (0x5C6608): `g_debugPixel16`: the debug font's colour made a 16-bit pixel
    /// (`debug::draw`).
    pub debug_pixel_16: Dword,
    /// +0x8 (0x5C660C): `g_debugPixel32`: the same, 32-bit.
    pub debug_pixel_32: Dword,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece: a static of its own since 2026-10-29 (it left the race block, phase 13:
/// `race::store::LEFT`), with the image's first values.
impl TextState {
    pub const fn new() -> Self {
        crate::pointers::cells_of_bytes(crate::race::store::first_values::<{ END - START }>(START))
    }
}

/// IDA 0x5C6604, the first byte of the piece.
pub const START: usize = 0x5C6604;
/// IDA 0x5C6610, the end of the piece.
pub const END: usize = 0x5C6610;

const _: () = assert!(END - START == core::mem::size_of::<TextState>());

/// Where IDA 0x5C6604 is now (the piece's static).
#[cfg(test)]
pub fn piece_addr() -> u32 {
    crate::pointers::game_address(std::ptr::from_ref(&race().frame.text_state))
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(&race().frame.text_state, START, END);
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
