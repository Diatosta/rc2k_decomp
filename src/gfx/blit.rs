//! The blitters' variables (the clip rectangle and the skips of the visible part). The rescaler's
//! own variables follow (`gfx::rescale_picture::RESCALE`).

use std::cell::Cell;

/// The clip rectangle (all four edges inclusive).
pub struct ClipRect {
    /// `Blitter.clip.left`.
    pub left: Cell<u32>,
    /// `Blitter.clip.right`.
    pub right: Cell<u32>,
    /// `Blitter.clip.top`.
    pub top: Cell<u32>,
    /// `Blitter.clip.bottom`.
    pub bottom: Cell<u32>,
}

pub struct Blitter {
    /// The clip rectangle.
    pub clip: ClipRect,
    /// Pixels clipped off on the left.
    pub left_skip: Cell<u32>,
    /// Pixels of each row inside the clip.
    pub visible_width: Cell<u32>,
    /// Rows clipped off at the top.
    pub top_skip: Cell<u32>,
    /// Visible rows (counted down while drawing).
    pub rows_left: Cell<u32>,
}

impl Blitter {
    /// The blitters' variables (a screen of 800 x 600).
    pub const fn new() -> Self {
        Blitter {
            clip: ClipRect {
                left: Cell::new(0),
                right: Cell::new(0x31F),
                top: Cell::new(0),
                bottom: Cell::new(0x257),
            },
            left_skip: Cell::new(0),
            visible_width: Cell::new(0),
            top_skip: Cell::new(0),
            rows_left: Cell::new(0),
        }
    }
}
