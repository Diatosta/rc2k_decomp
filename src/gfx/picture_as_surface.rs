//! `gfx::picture_as_surface` (0x405FE7): makes the picture buffer the surface that the
//! blitters draw on, so that sprites can be drawn into a picture.
//!
//! In:  nothing
//! Out: every register preserved
//! From `g_pictureFlags`: the low byte names the size (the table `dword_4F4E50`: 1 = 640 x
//! 480, 2 = 800 x 600, 4 = 960 x 720, 8 = 1024 x 768, 0x10 = 1152 x 864, 0x20 = 1280 x 1024,
//! 0x40 = 1600 x 1200, 0x80 = 1920 x 1280; anything else is fatal error 0x80A19091), bits 8
//! to 11 the pixels (the table `dword_4F4E40`: 0x200 = 16 bits, 0x800 = 32; anything else
//! is fatal error 0x80A19092). Into the surface block (`g_surface`, 0x4F4998) go the
//! width, the height, the bits and bytes of a pixel, the bytes of a row (also as the
//! pitch, with no gap after a row, and as dwords), the picture buffer as the pixels and
//! `g_surfaceFormat` as the format; the block's corner is left as it is. Then the clip
//! rectangle is set to the block's rectangle (`set_clip`).
//! Five callers, `menu::load_background` among them.
//! The name is a reading.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::{FatalError, Refused};
use crate::framework::rva;
use crate::gfx::block::{game_addr, game_read, game_write};
use crate::gfx::modes::Slot as ModeSlot;
use crate::gfx::picture::{FLAGS_RVA, PICTURE_RVA};
use crate::gfx::pixel_format::surface_format_addr;
use crate::gfx::set_clip::set_clip;
use crate::system::find_entry::entries;

// ---- game data ----

/// unk_4F4998: the surface block, twelve dwords: x, y, width, height, bits a pixel, bytes
/// a pixel, bytes of a row's pixels, dwords of a row, the pixel format, the pitch, the
/// pixels, the gap after a row.
pub const SURFACE_RVA: usize = rva(0x4F4998);
pub const SURFACE_DWORDS: usize = 12;
const SIZES_RVA: ModeSlot = ModeSlot::at(0x4F4E50); // dword_4F4E50: key, width and height as words
const SIZE_COUNT: u32 = 8;
const PIXEL_KINDS_RVA: ModeSlot = ModeSlot::at(0x4F4E40); // dword_4F4E40: key, bits
const PIXEL_KIND_COUNT: u32 = 2;

// ---- Rust version ----

/// The surface block for a picture with these flags, from its third dword on (the corner
/// is not touched).
pub unsafe fn surface_of(flags: u32, pixels: u32) -> Result<[u32; SURFACE_DWORDS - 2], Refused> {
    // Converted: `rc2k_core::gfx::screen_modes::picture_surface`.
    unsafe {
        rc2k_core::gfx::screen_modes::picture_surface(
            flags,
            entries(game_addr(SIZES_RVA) as u32, SIZE_COUNT),
            entries(game_addr(PIXEL_KINDS_RVA) as u32, PIXEL_KIND_COUNT),
            surface_format_addr() as u32,
            pixels,
        )
    }
    .map_err(|e| {
        FatalError {
            code: e.code(),
            message: "",
        }
        .into()
    })
}

pub unsafe fn picture_as_surface() -> Result<(), Refused> {
    unsafe {
        let block = surface_of(game_read(FLAGS_RVA), game_read(PICTURE_RVA))?;
        game_write(SURFACE_RVA + 8, block);
        set_clip(game_addr(SURFACE_RVA) as *const [u32; 4])?;
        Ok(())
    }
}

// ---- self-test: a copy of the original that reads the flags from a test variable, writes a
// surface block of the test's and hands the clip rectangle to a stand-in; the real surface
// and clip rectangle are what the game's drawing reads all the time ----
