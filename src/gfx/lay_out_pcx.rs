//! `gfx::lay_out_pcx` (0x411A82): works out from a PCX header how the picture is to be
//! decoded, and converts its palette.
//!
//! In:  nothing
//! Out: every register preserved
//! From the header at `dword_51B5F4`: the planes and the bytes of a plane's row; the
//! picture's width and height (right - left + 1, bottom - top + 1, as words), and those cut
//! to the picture buffer's size (`g_pictureWidth`, `g_pictureHeight`) and to the limits
//! `dword_51B974` / `dword_51B978`. Pixels are 8 bits when bit 8 of `g_pictureFlags` is set,
//! else 32. The stride from one decoded row to the next comes from `g_pictureRowBytes`:
//! 0xFFFFFFFF = a screen's width of pixels (800), 0xFFFFFFFE = the buffer's width of
//! pixels, 0xFFFFFFFD = the decoded row itself, anything else = that many bytes. The rows
//! go to `g_pictureBuffer`. Last, `palette_to_dwords` on the file's palette.
//! All of it lands in the variables listed in `gfx/pcx.rs`. Three callers, the PCX readers.
//! The high half of the two sizes is what the instruction before left there (16-bit moves
//! into eax), which the Rust version follows.
//! The name is a reading.

use crate::errors::BadInput;
use crate::gfx::decoder::DECODER;
use crate::gfx::palette_to_dwords::palette_to_dwords;
use crate::gfx::pcx::PCX;
use crate::gfx::picture_store::PICTURE;

// ---- Rust version ----

/// Converted: `rc2k_core::gfx::pcx::lay_out`, on the header and the variables in game memory.
/// Returns the layout it stored.
pub fn lay_out_pcx() -> Result<rc2k_core::gfx::pcx::Layout, BadInput> {
    let target = rc2k_core::gfx::pcx::Target {
        width: PICTURE.width.get(),
        height: PICTURE.height.get(),
        width_limit: PCX.width_limit.get(),
        height_limit: PCX.height_limit.get(),
        flags: PICTURE.flags.get(),
        row_bytes: PICTURE.row_bytes.get(),
    };
    let l = rc2k_core::gfx::pcx::lay_out(&PCX.header.read(), &target);
    for (at, value) in [
        (&PCX.planes, l.planes),
        (&PCX.line_bytes, l.line_bytes),
        (&PCX.file_width, l.file_width),
        (&DECODER.width, l.width),
        (&PCX.file_height, l.file_height),
        (&DECODER.height, l.height),
        (&DECODER.pixel_bits, l.pixel_bits),
        (&DECODER.pixel_bytes, l.pixel_bytes),
        (&DECODER.row_bytes, l.row_bytes),
        (&DECODER.row_dwords, l.row_dwords),
        (&DECODER.stride, l.stride),
        (&DECODER.row_gap, l.row_gap),
    ] {
        at.set(value);
    }
    DECODER.pixels.set(PICTURE.buffer.get());
    // Neither is null: both are the game's own tables.
    palette_to_dwords(
        PCX.file_palette.addr(),
        crate::pointers::game_address(DECODER.palette_ptr()),
    )?;
    Ok(l)
}
