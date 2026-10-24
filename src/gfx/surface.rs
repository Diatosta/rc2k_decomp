//! The global drawing surface and clip rectangle that the game's blitters read.

use crate::framework::rva;
use crate::gfx::blit::Slot as BlitSlot;
use crate::gfx::block::game_read;

pub const BITS_PER_PIXEL_RVA: usize = rva(0x4F49A8); // g_surfaceBitsPerPixel
pub const BYTES_PER_PIXEL_RVA: usize = rva(0x4F49AC); // g_surfaceBytesPerPixel
pub const PITCH_RVA: usize = rva(0x4F49BC); // g_surfacePitch
pub const BASE_RVA: usize = rva(0x4F49C0); // g_surfacePixels
pub const CLIP_LEFT_RVA: BlitSlot = BlitSlot::at(0x4F503C); // g_clipLeft
pub const CLIP_RIGHT_RVA: BlitSlot = BlitSlot::at(0x4F5040); // g_clipRight
pub const CLIP_TOP_RVA: BlitSlot = BlitSlot::at(0x4F5044); // g_clipTop
pub const CLIP_BOTTOM_RVA: BlitSlot = BlitSlot::at(0x4F5048); // g_clipBottom

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
    unsafe {
        Clip {
            left: game_read(CLIP_LEFT_RVA),
            right: game_read(CLIP_RIGHT_RVA),
            top: game_read(CLIP_TOP_RVA),
            bottom: game_read(CLIP_BOTTOM_RVA),
        }
    }
}
