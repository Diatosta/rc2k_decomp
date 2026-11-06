//! The progress bar's variables, out of RAL.EXE's image (phase 5; typed in phase 7,
//! `notes/conversion.md`): IDA 0x755224 .. 0x75523C. Set by the callers of
//! `gfx::blit_bar` / `gfx::blit_bar_marker` (`blit_bar::Bar` reads and writes the six).

use crate::session::Dword;

/// IDA's address of the block's first byte.
pub const START: usize = 0x755224;
/// The block's length.
pub const LEN: usize = 0x75523C - START;

/// The bar being drawn.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct BarVars {
    /// `dword_755224`: the bar's x (`blit_bar_marker` moves it to the marker).
    pub x: Dword,
    /// `dword_755228`: the bar's y.
    pub y: Dword,
    /// `dword_75522C`: the bar's length at 100 percent, in pixels.
    pub width: Dword,
    /// `dword_755230`: the fill's sprite (the marker's for `blit_bar_marker`).
    pub fill_glyph: Dword,
    /// `dword_755234`: the end piece's sprite.
    pub end_glyph: Dword,
    /// `dword_755238`: the percentage.
    pub percent: Dword,
}

impl BarVars {
    /// The bar; the first values are the image's.
    pub const fn new() -> Self {
        BarVars {
            x: Dword::new(0),
            y: Dword::new(0),
            width: Dword::new(0),
            fill_glyph: Dword::new(0),
            end_glyph: Dword::new(0),
            percent: Dword::new(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(
            &crate::gfx::owner::gfx().drawing.bar,
            START,
            START + LEN,
        );
    }

    #[test]
    fn moved_knows_the_block() {
        assert_eq!(
            crate::moved::place(START),
            crate::gfx::owner::gfx().drawing.bar.x.addr() as usize
        );
        assert_eq!(
            crate::moved::place(0x755238),
            crate::gfx::owner::gfx().drawing.bar.percent.addr() as usize
        );
    }
}
