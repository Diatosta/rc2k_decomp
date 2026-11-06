//! `gfx::lay_out_png` (0x41CCF7): sets the layout variables for a picture `decode_png` has
//! just decoded.
//!
//! In:  nothing
//! Out: every register preserved
//! The decoded width and height are the info block's (`buf`, 0x93E8B0, a `PngInfo`); pixels
//! are 8 bits when bit 8 of `g_pictureFlags` is set, else 32; a row follows the one before
//! without a gap; the rows are at `g_pictureBuffer`. Last, `palette_to_dwords` on the info
//! block's palette, whether the file had one or not. The planes, the bytes of a plane's row
//! and the file's own size, which `lay_out_pcx` sets, are left alone. One caller,
//! `gfx::read_png`. The name is a reading.

use crate::errors::BadInput;
use crate::gfx::owner::gfx;
use crate::gfx::palette_to_dwords::palette_to_dwords;
use crate::gfx::png::{PngInfo, info_dword};
use crate::system::owner::system;

// ---- Rust version ----

pub fn lay_out_png() -> Result<(), BadInput> {
    let pictures = &gfx().pictures;
    use core::mem::offset_of;
    // Converted: `rc2k_core::gfx::png::layout`.
    let layout = rc2k_core::gfx::png::layout(
        info_dword(offset_of!(PngInfo, width)),
        info_dword(offset_of!(PngInfo, height)),
        pictures.picture.flags.get(),
    );
    for (at, value) in [
        &pictures.decoder.width,
        &pictures.decoder.height,
        &pictures.decoder.pixel_bits,
        &pictures.decoder.pixel_bytes,
        &pictures.decoder.row_bytes,
        &pictures.decoder.row_dwords,
        &pictures.decoder.stride,
        &pictures.decoder.row_gap,
    ]
    .into_iter()
    .zip(layout)
    {
        at.set(value);
    }
    pictures.decoder.pixels.set(pictures.picture.buffer.get());
    // Neither is null: both are the game's own tables.
    palette_to_dwords(
        crate::pointers::GamePtr::to(&system().work.files.buffer)
            .offset(offset_of!(PngInfo, palette) as u32),
        crate::pointers::GamePtr::to(&pictures.decoder.palette),
    )?;
    Ok(())
}
