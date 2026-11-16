//! Sets the layout variables for a picture `decode_png` has just decoded.
//! The decoded width and height are the info record's (`gfx::png::PngInfo`); pixels are 8 bits when
//! bit 8 of the picture flags is set, else 32; a row follows the one before without a gap; the rows
//! are at the picture buffer. Last, `palette_to_dwords` on the info block's palette, whether the
//! file had one or not. The planes, the bytes of a plane's row and the file's own size, which
//! `lay_out_pcx` sets, are left alone.

use crate::errors::BadInput;
use crate::gfx::palette_to_dwords::palette_bytes_to_dwords;
use crate::gfx::png::PngInfo;

pub fn lay_out_png(gfx: &crate::gfx::owner::Gfx, info: &PngInfo) -> Result<(), BadInput> {
    let pictures = &gfx.pictures;
    let layout = rc2k_core::gfx::png::layout(info.width, info.height, pictures.picture.flags.get());
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
    palette_bytes_to_dwords(&info.palette, &pictures.decoder.palette)?;
    Ok(())
}
