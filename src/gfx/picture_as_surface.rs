//! Makes the picture buffer the surface that the blitters draw on, so that sprites can be drawn
//! into a picture. From the picture flags: the low byte names the size (1 = 640 x 480, 2 = 800 x
//! 600, 4 = 960 x 720, 8 = 1024 x 768, 0x10 = 1152 x 864, 0x20 = 1280 x 1024, 0x40 = 1600 x 1200,
//! 0x80 = 1920 x 1280; anything else is a fatal error), bits 8 to 11 the pixels (0x200 = 16 bits,
//! 0x800 = 32; anything else is a fatal error). Into the surface block go the width, the height,
//! the bits and bytes of a pixel, the bytes of a row (also as the pitch, with no gap after a row,
//! and as dwords), the picture buffer as the pixels and the surface format; the block's corner is
//! left as it is. Then the clip rectangle is set to the block's rectangle (`set_clip`).

use crate::errors::{FatalError, Refused};
use crate::gfx::modes::Modes;
use crate::gfx::set_clip::set_clip;

// `Gfx.drawing.draw_surface` is the surface block, twelve dwords: x, y, width, height, bits a pixel, bytes a
// pixel, bytes of a row's pixels, dwords of a row, the pixel format, the pitch, the pixels, the gap
// after a row.
const SURFACE_DWORDS: usize = 12;
const SIZE_COUNT: u32 = 8;
const PIXEL_KIND_COUNT: u32 = 2;

/// The surface block for a picture with these flags, from its third dword on (the corner is not
/// touched).
fn surface_of(
    modes: &Modes,
    flags: u32,
    pixels: u32,
) -> Result<[u32; SURFACE_DWORDS - 2], Refused> {
    rc2k_core::gfx::screen_modes::picture_surface(
        flags,
        modes.entries(Modes::SIZES_AT, SIZE_COUNT),
        modes.entries(Modes::PIXEL_KINDS_AT, PIXEL_KIND_COUNT),
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

pub fn picture_as_surface(gfx: &crate::gfx::owner::Gfx) -> Result<(), Refused> {
    // The surface is the picture buffer by its number (`gfx::surface::Target::Picture`),
    // the surface's pixels 0 (no address).
    let block = surface_of(&gfx.display.modes, gfx.pictures.picture.flags.get(), 0)?;
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
    gfx.drawing.on_picture.store(
        gfx.pictures.picture.buffer.get(),
        std::sync::atomic::Ordering::Relaxed,
    );
    s.row_gap.set(block[9]);
    set_clip([s.x.get(), s.y.get(), s.width.get(), s.height.get()], gfx);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every listed size with each listed pixel kind gives a surface of that size.
    #[test]
    fn every_listed_size_gives_a_surface() {
        let modes = Modes::new();
        for size in 0..SIZE_COUNT {
            for kind in [0x200, 0x800] {
                let block = surface_of(&modes, (1 << size) | kind, 0);
                assert!(block.is_ok(), "size bit {size}, pixels {kind:#x}");
            }
        }
        assert!(surface_of(&modes, 0x100 | 0x200, 0).is_err());
    }
}
