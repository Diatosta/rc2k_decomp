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
use crate::files::close_stream::close_stream;
use crate::files::handle::{read, seek};
use crate::files::open_stream::open_stream;
use crate::files::read_stream_block::read_stream_block;
use crate::files::read_stream_byte::read_stream_byte;
use crate::gfx::decoder::DECODER;
use crate::gfx::lay_out_pcx::lay_out_pcx;
use crate::gfx::pcx::PCX;
use crate::gfx::pcx::{FILE_PALETTE_LEN, HEADER_LEN, stream};
use crate::gfx::picture_store::PICTURE;

// ---- Rust version ----

/// The part between the open and the close: whether it worked.
unsafe fn read_rows() -> Result<bool, BadInput> {
    unsafe {
        let stream = stream();
        let file = (*stream).file;
        if seek(file, -(FILE_PALETTE_LEN as i32), 2).is_none() {
            return Ok(false);
        }
        let palette_bytes = crate::pointers::host(PCX.file_palette.addr());
        if !read(file, palette_bytes, FILE_PALETTE_LEN as u32).1 {
            return Ok(false);
        }
        if seek(file, 0, 0).is_none() {
            return Ok(false);
        }
        let header = crate::pointers::host(PCX.header.addr());
        if read_stream_block(stream, header, HEADER_LEN as u32)?.1 {
            return Ok(false);
        }
        PICTURE.flags.set(0x800u32);
        let layout = lay_out_pcx()?;
        // Converted: `rc2k_core::gfx::pcx::decode_8bit`.
        let palette: [u32; 256] = DECODER.palette();
        let pixels: u32 = DECODER.pixels.get();
        rc2k_core::gfx::pcx::decode_8bit(
            &layout,
            &palette,
            |eax| read_stream_byte(stream, eax).map(|r| r.eax),
            |to, pixel| {
                (crate::pointers::host(pixels.wrapping_add(to)) as *mut u32).write_unaligned(pixel)
            },
        )?
        .ok_or(BadInput("a PCX picture without rows"))?;
        Ok(true)
    }
}

pub unsafe fn read_pcx_8bit(path: *const u8) -> Result<bool, BadInput> {
    unsafe {
        if !open_stream(stream(), path, 0)? {
            return Ok(false);
        }
        let rows = read_rows();
        let closed = close_stream(stream())?;
        Ok(rows? && closed)
    }
}
