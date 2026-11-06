//! The blitters' variables (the clip rectangle and the skips of the visible part), out of
//! RAL.EXE's image (phase 5; typed in phase 7, `notes/conversion.md`): IDA 0x4F503C ..
//! 0x4F505C. The rescaler's own variables follow (`gfx::rescale_picture::RESCALE`).

use crate::session::Dword;

/// IDA's address of the block's first byte.
pub const START: usize = 0x4F503C;
/// The block's length.
pub const LEN: usize = 0x4F505C - START;

/// The clip rectangle (all four edges inclusive).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ClipRect {
    /// `g_clipLeft` (`dword_4F503C`).
    pub left: Dword,
    /// `g_clipRight` (`dword_4F5040`).
    pub right: Dword,
    /// `g_clipTop` (`dword_4F5044`).
    pub top: Dword,
    /// `g_clipBottom` (`dword_4F5048`).
    pub bottom: Dword,
}

/// The blitters' variables.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Blitter {
    /// The clip rectangle.
    pub clip: ClipRect,
    /// `g_blitLeftSkip`: pixels clipped off on the left.
    pub left_skip: Dword,
    /// `g_blitVisibleWidth`: pixels of each row inside the clip.
    pub visible_width: Dword,
    /// `g_blitTopSkip`: rows clipped off at the top.
    pub top_skip: Dword,
    /// `g_blitRowsLeft`: visible rows (counted down while drawing).
    pub rows_left: Dword,
}

impl Blitter {
    /// The blitters' variables; the first values are the image's (a screen of 800 x 600).
    pub const fn new() -> Self {
        Blitter {
            clip: ClipRect {
                left: Dword::new(0),
                right: Dword::new(0x31F),
                top: Dword::new(0),
                bottom: Dword::new(0x257),
            },
            left_skip: Dword::new(0),
            visible_width: Dword::new(0),
            top_skip: Dword::new(0),
            rows_left: Dword::new(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(
            &crate::gfx::owner::gfx().drawing.blit,
            START,
            START + LEN,
        );
    }

    #[test]
    fn moved_knows_the_block() {
        assert_eq!(offset_of!(Blitter, left_skip), 0x4F504C - START);
        assert_eq!(offset_of!(Blitter, rows_left), 0x4F5058 - START);
        assert_eq!(
            crate::moved::place(START),
            crate::gfx::owner::gfx().drawing.blit.clip.left.addr() as usize
        );
        assert_eq!(
            crate::moved::place(0x4F5058),
            crate::gfx::owner::gfx().drawing.blit.rows_left.addr() as usize
        );
    }
}
