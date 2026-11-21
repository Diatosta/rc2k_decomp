//! The progress bar's variables, set by the callers of `gfx::blit_bar` / `gfx::blit_bar_marker`
//! (`blit_bar::Bar` reads and writes the six).

use std::cell::Cell;

/// The bar being drawn.
pub struct BarVars {
    /// The bar's x (`blit_bar_marker` moves it to the marker).
    pub x: Cell<u32>,
    /// The bar's y.
    pub y: Cell<u32>,
    /// The bar's length at 100 percent, in pixels.
    pub width: Cell<u32>,
    /// The fill's sprite (the marker's for `blit_bar_marker`).
    pub fill_glyph: Cell<u32>,
    /// The end piece's sprite.
    pub end_glyph: Cell<u32>,
    /// The percentage.
    pub percent: Cell<u32>,
}

impl BarVars {
    /// The bar.
    pub const fn new() -> Self {
        BarVars {
            x: Cell::new(0),
            y: Cell::new(0),
            width: Cell::new(0),
            fill_glyph: Cell::new(0),
            end_glyph: Cell::new(0),
            percent: Cell::new(0),
        }
    }
}
