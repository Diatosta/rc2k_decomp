//! The progress bar's variables, set by the callers of `gfx::blit_bar` / `gfx::blit_bar_marker`
//! (`blit_bar::Bar` reads and writes the six).

use crate::session::PlainDword;

/// The bar being drawn.
#[derive(rc2k_derive::Cells)]
pub struct BarVars {
    /// The bar's x (`blit_bar_marker` moves it to the marker).
    pub x: PlainDword,
    /// The bar's y.
    pub y: PlainDword,
    /// The bar's length at 100 percent, in pixels.
    pub width: PlainDword,
    /// The fill's sprite (the marker's for `blit_bar_marker`).
    pub fill_glyph: PlainDword,
    /// The end piece's sprite.
    pub end_glyph: PlainDword,
    /// The percentage.
    pub percent: PlainDword,
}

impl BarVars {
    /// The bar.
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
