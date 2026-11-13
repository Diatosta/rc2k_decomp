//! The progress bar's variables, out of RAL.EXE's image: IDA 0x755224 .. 0x75523C. Set by the
//! callers of `gfx::blit_bar` / `gfx::blit_bar_marker` (`blit_bar::Bar` reads and writes the six).

use crate::session::PlainDword;

/// The bar being drawn.
#[derive(rc2k_derive::Cells)]
pub struct BarVars {
    /// `dword_755224`: the bar's x (`blit_bar_marker` moves it to the marker).
    pub x: PlainDword,
    /// `dword_755228`: the bar's y.
    pub y: PlainDword,
    /// `dword_75522C`: the bar's length at 100 percent, in pixels.
    pub width: PlainDword,
    /// `dword_755230`: the fill's sprite (the marker's for `blit_bar_marker`).
    pub fill_glyph: PlainDword,
    /// `dword_755234`: the end piece's sprite.
    pub end_glyph: PlainDword,
    /// `dword_755238`: the percentage.
    pub percent: PlainDword,
}

impl BarVars {
    /// The bar; the first values are the image's.
    pub const fn new() -> Self {
        BarVars {
            x: PlainDword::new(0),
            y: PlainDword::new(0),
            width: PlainDword::new(0),
            fill_glyph: PlainDword::new(0),
            end_glyph: PlainDword::new(0),
            percent: PlainDword::new(0),
        }
    }
}
