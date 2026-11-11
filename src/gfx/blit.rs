//! The blitters' variables (the clip rectangle and the skips of the visible part), out of
//! RAL.EXE's image: IDA 0x4F503C ..
//! 0x4F505C. The rescaler's own variables follow (`gfx::rescale_picture::RESCALE`).

use crate::session::PlainDword;

/// IDA's address of the block's first byte.
pub const START: usize = 0x4F503C;
/// The block's length.
pub const LEN: usize = 0x4F505C - START;

/// The clip rectangle (all four edges inclusive).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ClipRect {
    /// `g_clipLeft` (`Blitter.clip.left`).
    pub left: PlainDword,
    /// `g_clipRight` (`Blitter.clip.right`).
    pub right: PlainDword,
    /// `g_clipTop` (`Blitter.clip.top`).
    pub top: PlainDword,
    /// `g_clipBottom` (`Blitter.clip.bottom`).
    pub bottom: PlainDword,
}

/// The blitters' variables.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Blitter {
    /// The clip rectangle.
    pub clip: ClipRect,
    /// `g_blitLeftSkip`: pixels clipped off on the left.
    pub left_skip: PlainDword,
    /// `g_blitVisibleWidth`: pixels of each row inside the clip.
    pub visible_width: PlainDword,
    /// `g_blitTopSkip`: rows clipped off at the top.
    pub top_skip: PlainDword,
    /// `g_blitRowsLeft`: visible rows (counted down while drawing).
    pub rows_left: PlainDword,
}

impl Blitter {
    /// The blitters' variables; the first values are the image's (a screen of 800 x 600).
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gfx::owner::gfx;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(&gfx().drawing.blit, START, START + LEN);
    }

    #[test]
    fn moved_knows_the_block() {
        let drawing = &gfx().drawing;
        assert_eq!(offset_of!(Blitter, left_skip), 0x4F504C - START);
        assert_eq!(offset_of!(Blitter, rows_left), 0x4F5058 - START);
        assert_eq!(
            crate::moved::place(START),
            drawing.blit.clip.left.addr() as usize
        );
        assert_eq!(
            crate::moved::place(0x4F5058),
            drawing.blit.rows_left.addr() as usize
        );
    }
}
