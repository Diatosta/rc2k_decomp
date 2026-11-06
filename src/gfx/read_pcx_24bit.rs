//! `gfx::read_pcx_24bit` (0x411D57): decodes a PCX file of three planes, 8 bits each, into
//! the picture buffer as 32-bit pixels.
//!
//! In:  eax = path
//! Out: carry = failed; every register preserved
//! Opens the file as the stream `dword_5195E4`, reads the header through it, sets
//! `g_pictureFlags` to 0x800 and calls `lay_out_pcx`. Then the rows. A row in the file is
//! one plane after the other, each the header's count of bytes; plane n's bytes go to byte
//! `planes - 1 - n` of the pixels (so the first plane is the third byte), those within the
//! decoded width. Then the fourth byte of every pixel of the row is set to 0, and the
//! destination moves on by the stride. A run goes on across the ends of planes and rows,
//! but the byte it repeats is 0 from the end of a row on (the zeroing leaves al 0). What
//! `read_stream_byte` says about the end of the file is not looked at.
//! It fails if the open, the header's read or the close fails.
//! Two callers, `gfx::read_pcx` and `sub_4119AF`. Refuses a decoded width or height of 0 (the
//! original counts through all 32 bits) and what `open_stream` refuses. The name is a reading.

use crate::errors::BadInput;
use crate::files::read_stream_byte::{ByteRead, read_stream_byte};
use crate::files::stream::Stream;
use crate::gfx::lay_out_pcx::lay_out_pcx;
use crate::gfx::owner::gfx;
use crate::gfx::pcx::{on_stream, read_stream_header};
use crate::gfx::picture::{put_or_end, with_buffer};

// ---- Rust version ----

/// The part between the open and the close: whether it worked.
fn read_rows(stream: &mut Stream) -> Result<bool, BadInput> {
    let pictures = &gfx().pictures;
    if !read_stream_header(stream)? {
        return Ok(false);
    }
    pictures.picture.flags.set(0x800u32);
    let layout = lay_out_pcx()?;
    // Converted: `rc2k_core::gfx::pcx::decode_planes`.
    with_buffer(pictures.decoder.pixels.get(), |pixels| {
        rc2k_core::gfx::pcx::decode_planes(
            &layout,
            || read_stream_byte(stream).map(ByteRead::value),
            |to, byte| put_or_end(pixels, to as usize, &[byte]),
        )
    })??
    .ok_or(BadInput("a PCX picture without rows or columns"))?;
    Ok(true)
}

pub fn read_pcx_24bit(path: &[u8]) -> Result<bool, BadInput> {
    on_stream(path, read_rows)
}
