//! The HUD's view strips' places (sizes, tops, lefts), used by `hud::draw_view_strip` only.

use std::cell::Cell;

/// The view strips' places.
pub struct Strips {
    /// The strip being drawn: top, height, left, width.
    pub sizes: [Cell<u32>; 4],
    /// The strips' tops (12; the entry is picked by the view and the flat layout).
    pub tops: [u32; 12],
    /// The strips' lefts (6, one a view).
    pub lefts: [u32; 6],
}

impl Strips {
    /// The places.
    pub const fn new() -> Self {
        Strips {
            sizes: [const { Cell::new(0) }; 4],
            tops: [
                0x80, 0xb1, 0x53, 0, 0xb1, 0x18e, 0x60, 0x91, 0x53, 0, 0x91, 0x17e,
            ],
            lefts: [0xdc, 0x14, 0xdc, 0, 0x1a4, 0xdc],
        }
    }
}
