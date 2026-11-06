//! `gfx::read_pcx_8bit` (0x411C6B): decodes a PCX file of one plane, 8 bits a pixel with a
//! palette, into the picture buffer as 32-bit pixels.
//!
//! In:  eax = path
//! Out: carry = failed; every register preserved
//! Opens the file as the stream `dword_5195E4`; reads the palette, the last 0x300 bytes of
//! the file, with a seek and a plain read on the stream's handle, and seeks back to the
//! start; reads the header through the stream; sets `g_pictureFlags` to 0x800 and calls
//! `lay_out_pcx`. Then the rows: for each of the row's bytes (the header's count, which may
//! be more than the width) the next palette number, whose dword is stored if the byte is
//! within the decoded width. A run goes on across the end of a row. After a row the
//! destination moves on by the row gap. What `read_stream_byte` says about the end of the
//! file is not looked at: a file that ends early is filled up with palette entry 0.
//! It fails if the open, a seek, the palette's read, the header's read or the close fails.
//! A count byte of 0xC0, a run of none, makes a run of 2^32: the rest of the picture.
//! One caller, `gfx::read_pcx`. Refuses a decoded height of 0 (the original counts rows through
//! all 32 bits) and what `open_stream` refuses. The name is a reading.

use crate::errors::BadInput;
use crate::files::handle::{read_into, seek};
use crate::files::read_stream_byte::{ByteRead, read_stream_byte};
use crate::files::stream::Stream;
use crate::gfx::lay_out_pcx::lay_out_pcx;
use crate::gfx::owner::gfx;
use crate::gfx::pcx::{FILE_PALETTE_LEN, on_stream, read_stream_header};
use crate::gfx::picture::{put_or_end, with_buffer};

// ---- Rust version ----

/// The file's palette read with a seek and a plain read on the stream's handle, and the
/// handle back at the start: whether it worked.
pub fn read_file_palette(stream: &Stream) -> bool {
    let file = stream.file;
    seek(file, -(FILE_PALETTE_LEN as i32), 2).is_some()
        && read_into(file, &gfx().pictures.pcx.file_palette).1
        && seek(file, 0, 0).is_some()
}

/// The part between the open and the close: whether it worked.
fn read_rows(stream: &mut Stream) -> Result<bool, BadInput> {
    let pictures = &gfx().pictures;
    if !read_file_palette(stream) || !read_stream_header(stream)? {
        return Ok(false);
    }
    pictures.picture.flags.set(0x800u32);
    let layout = lay_out_pcx()?;
    // Converted: `rc2k_core::gfx::pcx::decode_8bit`.
    let palette: [u32; 256] = pictures.decoder.palette();
    with_buffer(pictures.decoder.pixels.get(), |pixels| {
        rc2k_core::gfx::pcx::decode_8bit(
            &layout,
            &palette,
            || read_stream_byte(stream).map(ByteRead::value),
            |to, pixel| put_or_end(pixels, to as usize, &pixel.to_le_bytes()),
        )
    })??
    .ok_or(BadInput("a PCX picture without rows"))?;
    Ok(true)
}

pub fn read_pcx_8bit(path: &[u8]) -> Result<bool, BadInput> {
    on_stream(path, read_rows)
}
