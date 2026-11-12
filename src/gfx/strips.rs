//! The HUD's view strips' places (sizes, tops, lefts), out of RAL.EXE's image: IDA 0x6D1870 ..
//! 0x6D18C8. Used by `hud::draw_view_strip` only.

use crate::gfx::plain_dwords;
use crate::session::PlainDword;

/// IDA's address of the block's first byte.
pub const START: usize = 0x6D1870;
/// The block's length.
pub const LEN: usize = 0x6D18C8 - START;

/// The view strips' places.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Strips {
    /// `dword_6D1870` .. `dword_6D187C`: the strip being drawn: top, height, left, width.
    pub sizes: [PlainDword; 4],
    /// `dword_6D1880`: the strips' tops (12; the entry is picked by the view and the flat layout).
    pub tops: [PlainDword; 12],
    /// `dword_6D18B0`: the strips' lefts (6, one a view).
    pub lefts: [PlainDword; 6],
}

impl Strips {
    /// The places; the first values are the image's.
    pub const fn new() -> Self {
        Strips {
            sizes: plain_dwords([0; 4]),
            tops: plain_dwords([
                0x80, 0xb1, 0x53, 0, 0xb1, 0x18e, 0x60, 0x91, 0x53, 0, 0x91, 0x17e,
            ]),
            lefts: plain_dwords([0xdc, 0x14, 0xdc, 0, 0x1a4, 0xdc]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::test_support::first_values::assert_block_starts_as_the_games(
            &crate::gfx::owner::GFX.unchecked().strips,
        );
    }

    #[test]
    fn moved_knows_the_block() {
        assert_eq!(offset_of!(Strips, tops), 0x10);
        assert_eq!(offset_of!(Strips, lefts), 0x40);
    }
}
