//! Works out from a PCX header how the picture is to be decoded, and converts its palette.
//!
//! From the header: the planes and the bytes of a plane's row; the picture's width and height
//! (right - left + 1, bottom - top + 1, as words), and those cut to the picture buffer's size and
//! to the limits `width_limit` / `height_limit`. Pixels are 8 bits when bit 8 of the picture
//! flags is set, else 32. The stride from one decoded row to the next comes from the picture row
//! bytes setting: 0xFFFFFFFF = a screen's width of pixels (800), 0xFFFFFFFE = the buffer's width
//! of pixels, 0xFFFFFFFD = the decoded row itself, anything else = that many bytes. The rows go
//! to the picture buffer. Last, `palette_bytes_to_dwords` on the file's palette. All of it lands in the
//! variables listed in `gfx/pcx.rs`.

use crate::errors::BadInput;
use crate::gfx::palette_to_dwords::palette_bytes_to_dwords;
use crate::gfx::pcx::FILE_PALETTE_LEN;

/// Returns the layout it stored.
pub(super) fn lay_out_pcx(
    gfx: &crate::gfx::owner::Gfx,
) -> Result<rc2k_core::gfx::pcx::Layout, BadInput> {
    let pictures = &gfx.pictures;
    let target = rc2k_core::gfx::pcx::Target {
        width: pictures.picture.width.get(),
        height: pictures.picture.height.get(),
        width_limit: pictures.pcx.width_limit.get(),
        height_limit: pictures.pcx.height_limit.get(),
        flags: pictures.picture.flags.get(),
        row_bytes: pictures.picture.row_bytes.get(),
    };
    let l = rc2k_core::gfx::pcx::lay_out(&pictures.pcx.header.read(), &target);
    for (at, value) in [
        (&pictures.pcx.planes, l.planes),
        (&pictures.pcx.line_bytes, l.line_bytes),
        (&pictures.pcx.file_width, l.file_width),
        (&pictures.decoder.width, l.width),
        (&pictures.pcx.file_height, l.file_height),
        (&pictures.decoder.height, l.height),
        (&pictures.decoder.pixel_bits, l.pixel_bits),
        (&pictures.decoder.pixel_bytes, l.pixel_bytes),
        (&pictures.decoder.row_bytes, l.row_bytes),
        (&pictures.decoder.row_dwords, l.row_dwords),
        (&pictures.decoder.stride, l.stride),
        (&pictures.decoder.row_gap, l.row_gap),
    ] {
        at.set(value);
    }
    pictures.decoder.pixels.set(pictures.picture.buffer.get());
    // Neither is null: both are the game's own tables.
    let palette = pictures
        .pcx
        .file_palette
        .read_at(0, FILE_PALETTE_LEN, "the file's palette")?;
    let palette: [u8; FILE_PALETTE_LEN] = palette.try_into().expect("a palette's length");
    palette_bytes_to_dwords(&palette, &pictures.decoder.palette)?;
    Ok(l)
}
