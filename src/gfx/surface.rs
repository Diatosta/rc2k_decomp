//! The global drawing surface and clip rectangle that the game's blitters read.

use crate::gfx::blit::BLIT;
use crate::gfx::screen_state::DRAW_SURFACE;

/// Where pixels go. `base` is an address; 0 means no surface is set.
#[derive(Clone, Copy, Debug)]
pub struct Surface {
    pub base: u32,
    /// Bytes from one row to the next.
    pub pitch: i32,
    pub bytes_per_pixel: i32,
    pub bits_per_pixel: u32,
}

/// The clip rectangle, all four edges inclusive.
#[derive(Clone, Copy, Debug)]
pub struct Clip {
    pub left: i32,
    pub right: i32,
    pub top: i32,
    pub bottom: i32,
}

pub fn surface() -> Surface {
    Surface {
        base: DRAW_SURFACE.pixels.get(),
        pitch: DRAW_SURFACE.pitch.get() as i32,
        bytes_per_pixel: DRAW_SURFACE.bytes_per_pixel.get() as i32,
        bits_per_pixel: DRAW_SURFACE.bits_per_pixel.get(),
    }
}

pub fn clip() -> Clip {
    Clip {
        left: BLIT.clip.left.get() as i32,
        right: BLIT.clip.right.get() as i32,
        top: BLIT.clip.top.get() as i32,
        bottom: BLIT.clip.bottom.get() as i32,
    }
}
