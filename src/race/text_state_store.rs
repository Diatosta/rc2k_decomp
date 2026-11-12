//! The frame text's state in the race block, 0x5C6604 .. 0x5C6610: `TextState.text_mode`, the original's
//! debug drawing colour number (IDA: read by `debug::fill_rect_16/32`, `debug::draw_glyph_16/32`, the
//! wear-bar and watch-page drawers, which pick the pixel from it), and the two dwords after it
//! (that colour as a 16-bit and as a 32-bit pixel, `g_debugPixel16/32`, read by the glyph
//! drawers only). `race::draw_frame_rate` sets the number to 2 each frame; the port's debug
//! overlay (`debug::ui`) reads none of the three, so they are written and never read here.

#[cfg(test)]
use crate::race::owner::race;
use crate::session::PlainDword;

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct TextState {
    /// +0x0 (0x5C6604): `dword_5C6604`: the original's debug drawing colour number (read by its fill-rectangle and glyph drawers); `race::draw_frame_rate` sets it to 2 before the frame-rate text. Nothing in the port reads it.
    pub text_mode: PlainDword,
    /// +0x4 (0x5C6608): `g_debugPixel16`: the debug font's colour made a 16-bit pixel
    /// (`debug::draw`).
    pub debug_pixel_16: PlainDword,
    /// +0x8 (0x5C660C): `g_debugPixel32`: the same, 32-bit.
    pub debug_pixel_32: PlainDword,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece, with the image's first values.
impl TextState {
    pub const fn new() -> Self {
        crate::cells::cells_of_bytes(crate::race::store::first_values::<{ END - START }>(START))
    }
}

/// IDA 0x5C6604, the first byte of the piece.
pub const START: usize = 0x5C6604;
/// IDA 0x5C6610, the end of the piece.
pub const END: usize = 0x5C6610;

const _: () = assert!(END - START == core::mem::size_of::<TextState>());

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_values_are_the_games() {
        crate::test_support::first_values::assert_block_starts_as_the_games(
            race().frame.text_state(),
        );
    }
}
