//! The global drawing surface and clip rectangle that the game's blitters read.

use crate::framework::rva;
use crate::gfx::blit::BLIT;
use crate::store::game_read;

pub const BITS_PER_PIXEL_RVA: usize = rva(0x4F49A8); // g_surfaceBitsPerPixel
pub const BYTES_PER_PIXEL_RVA: usize = rva(0x4F49AC); // g_surfaceBytesPerPixel
pub const PITCH_RVA: usize = rva(0x4F49BC); // g_surfacePitch
pub const BASE_RVA: usize = rva(0x4F49C0); // g_surfacePixels

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
    unsafe {
        Surface {
            base: game_read(BASE_RVA),
            pitch: game_read(PITCH_RVA),
            bytes_per_pixel: game_read(BYTES_PER_PIXEL_RVA),
            bits_per_pixel: game_read(BITS_PER_PIXEL_RVA),
        }
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
