//! The blitters' variables (the clip rectangle and the skips of the visible part). The rescaler's
//! own variables follow (`gfx::rescale_picture::RESCALE`).

use crate::session::PlainDword;

/// The clip rectangle (all four edges inclusive).
#[derive(rc2k_derive::Cells)]
pub struct ClipRect {
    /// `Blitter.clip.left`.
    pub left: PlainDword,
    /// `Blitter.clip.right`.
    pub right: PlainDword,
    /// `Blitter.clip.top`.
    pub top: PlainDword,
    /// `Blitter.clip.bottom`.
    pub bottom: PlainDword,
}

#[derive(rc2k_derive::Cells)]
pub struct Blitter {
    /// The clip rectangle.
    pub clip: ClipRect,
    /// Pixels clipped off on the left.
    pub left_skip: PlainDword,
    /// Pixels of each row inside the clip.
    pub visible_width: PlainDword,
    /// Rows clipped off at the top.
    pub top_skip: PlainDword,
    /// Visible rows (counted down while drawing).
    pub rows_left: PlainDword,
}

impl Blitter {
    /// The blitters' variables (a screen of 800 x 600).
    pub const fn new() -> Self {
        Blitter {
            clip: ClipRect {
                left: PlainDword::new(0),
                right: PlainDword::new(0x31F),
                top: PlainDword::new(0),
                bottom: PlainDword::new(0x257),
            },
            left_skip: PlainDword::new(0),
            visible_width: PlainDword::new(0),
            top_skip: PlainDword::new(0),
            rows_left: PlainDword::new(0),
        }
    }
}
