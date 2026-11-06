//! The HUD's view strips' places (sizes, tops, lefts), out of RAL.EXE's image (phase 5;
//! typed in phase 7, `notes/conversion.md`): IDA 0x6D1870 .. 0x6D18C8. Used by
//! `hud::draw_view_strip` only.

use crate::gfx::dwords;
use crate::session::Dword;

/// IDA's address of the block's first byte.
pub const START: usize = 0x6D1870;
/// The block's length.
pub const LEN: usize = 0x6D18C8 - START;

/// The view strips' places.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Strips {
    /// `dword_6D1870` .. `dword_6D187C`: the strip being drawn: top, height, left, width.
    pub sizes: [Dword; 4],
    /// `dword_6D1880`: the strips' tops (12; the entry is picked by the view and the flat layout).
    pub tops: [Dword; 12],
    /// `dword_6D18B0`: the strips' lefts (6, one a view).
    pub lefts: [Dword; 6],
}

impl Strips {
    /// The places; the first values are the image's.
    pub const fn new() -> Self {
        Strips {
            sizes: dwords([0; 4]),
            tops: dwords([
                0x80, 0xb1, 0x53, 0, 0xb1, 0x18e, 0x60, 0x91, 0x53, 0, 0x91, 0x17e,
            ]),
            lefts: dwords([0xdc, 0x14, 0xdc, 0, 0x1a4, 0xdc]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(&crate::gfx::owner::GFX.strips, START, START + LEN);
    }

    #[test]
    fn moved_knows_the_block() {
        assert_eq!(offset_of!(Strips, tops), 0x6D1880 - START);
        assert_eq!(offset_of!(Strips, lefts), 0x6D18B0 - START);
        assert_eq!(
            crate::moved::place(START),
            crate::gfx::owner::GFX.strips.sizes[0].addr() as usize
        );
        assert_eq!(
            crate::moved::place(0x6D18B0),
            crate::gfx::owner::GFX.strips.lefts[0].addr() as usize
        );
    }
}
