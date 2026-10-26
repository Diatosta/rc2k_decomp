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
use crate::gfx::decoder::DECODER;
use crate::gfx::palette_to_dwords::palette_to_dwords;
use crate::gfx::picture_store::PICTURE;
use crate::gfx::png::{PngInfo, info};

// ---- Rust version ----

pub unsafe fn lay_out_png() -> Result<(), BadInput> {
    unsafe {
        let info: *mut PngInfo = info();
        // Converted: `rc2k_core::gfx::png::layout`.
        let layout = rc2k_core::gfx::png::layout(
            (&raw const (*info).width).read_unaligned(),
            (&raw const (*info).height).read_unaligned(),
            PICTURE.flags.get(),
        );
        for (at, value) in [
            &DECODER.width,
            &DECODER.height,
            &DECODER.pixel_bits,
            &DECODER.pixel_bytes,
            &DECODER.row_bytes,
            &DECODER.row_dwords,
            &DECODER.stride,
            &DECODER.row_gap,
        ]
        .into_iter()
        .zip(layout)
        {
            at.set(value);
        }
        DECODER.pixels.set(PICTURE.buffer.get());
        // Neither is null: both are the game's own tables.
        palette_to_dwords(
            crate::pointers::game_address(&raw const (*info).palette),
            crate::pointers::game_address(DECODER.palette_ptr()),
        )?;
        Ok(())
    }
}
