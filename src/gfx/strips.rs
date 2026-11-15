//! The HUD's view strips' places (sizes, tops, lefts), used by `hud::draw_view_strip` only.

use crate::gfx::plain_dwords;
use crate::memory::session::PlainDword;

/// The view strips' places.
#[derive(rc2k_derive::Cells)]
pub struct Strips {
    /// The strip being drawn: top, height, left, width.
    pub sizes: [PlainDword; 4],
    /// The strips' tops (12; the entry is picked by the view and the flat layout).
    pub tops: [PlainDword; 12],
    /// The strips' lefts (6, one a view).
    pub lefts: [PlainDword; 6],
}

impl Strips {
    /// The places.
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
