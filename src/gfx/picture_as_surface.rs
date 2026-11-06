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
//! The registers above are the original's, for reference; a call by address goes through
//! `routes.rs`.

use crate::errors::{FatalError, Refused};
use crate::gfx::owner::gfx;
use crate::gfx::set_clip::set_clip;
use crate::system::find_entry::entries;

// ---- game data ----

// unk_4F4998 is `DRAW_SURFACE`'s x: the surface block, twelve dwords: x, y, width, height,
// bits a pixel, bytes a pixel, bytes of a row's pixels, dwords of a row, the pixel format,
// the pitch, the pixels, the gap after a row.
pub const SURFACE_DWORDS: usize = 12;
const SIZE_COUNT: u32 = 8;
const PIXEL_KIND_COUNT: u32 = 2;

// ---- Rust version ----

/// The surface block for a picture with these flags, from its third dword on (the corner
/// is not touched).
pub fn surface_of(flags: u32, pixels: u32) -> Result<[u32; SURFACE_DWORDS - 2], Refused> {
    // Converted: `rc2k_core::gfx::screen_modes::picture_surface`.
    rc2k_core::gfx::screen_modes::picture_surface(
        flags,
        entries(crate::gfx::modes::Modes::sizes_ptr(), SIZE_COUNT),
        entries(
            crate::gfx::modes::Modes::pixel_kinds_ptr(),
            PIXEL_KIND_COUNT,
        ),
        crate::gfx::pixel_format::SCREEN_FORMAT,
        pixels,
    )
    .map_err(|e| {
        FatalError {
            code: e.code(),
            message: "",
        }
        .into()
    })
}

pub fn picture_as_surface() -> Result<(), Refused> {
    let gfx = gfx();
    // The surface is the picture buffer by its number (`gfx::surface::Target::Picture`),
    // `g_surface.pixels` 0 (no address).
    let block = surface_of(gfx.pictures.picture.flags.get(), 0)?;
    let s = &gfx.drawing.draw_surface;
    s.width.set(block[0]);
    s.height.set(block[1]);
    s.bits_per_pixel.set(block[2]);
    s.bytes_per_pixel.set(block[3]);
    s.row_bytes.set(block[4]);
    s.row_dwords.set(block[5]);
    s.format.set(block[6]);
    s.pitch.set(block[7]);
    s.pixels.set(block[8]);
    crate::gfx::surface::ON_PICTURE.store(
        gfx.pictures.picture.buffer.get(),
        std::sync::atomic::Ordering::Relaxed,
    );
    s.row_gap.set(block[9]);
    set_clip([s.x.get(), s.y.get(), s.width.get(), s.height.get()]);
    Ok(())
}
