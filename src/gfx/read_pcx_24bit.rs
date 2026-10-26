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
use crate::files::close_stream::close_stream;
use crate::files::open_stream::open_stream;
use crate::files::read_stream_block::read_stream_block;
use crate::files::read_stream_byte::read_stream_byte;
use crate::gfx::decoder::DECODER;
use crate::gfx::lay_out_pcx::lay_out_pcx;
use crate::gfx::pcx::PCX;
use crate::gfx::pcx::{HEADER_LEN, stream};
use crate::gfx::picture_store::PICTURE;

// ---- Rust version ----

/// The part between the open and the close: whether it worked.
unsafe fn read_rows() -> Result<bool, BadInput> {
    unsafe {
        let stream = stream();
        let header = PCX.header.addr() as *mut u8;
        if read_stream_block(stream, header, HEADER_LEN as u32)?.1 {
            return Ok(false);
        }
        PICTURE.flags.set(0x800u32);
        let layout = lay_out_pcx()?;
        // Converted: `rc2k_core::gfx::pcx::decode_planes`.
        let pixels: u32 = DECODER.pixels.get();
        rc2k_core::gfx::pcx::decode_planes(
            &layout,
            |eax| read_stream_byte(stream, eax).map(|r| r.eax),
            |to, byte| (pixels.wrapping_add(to) as *mut u8).write(byte),
        )?
        .ok_or(BadInput("a PCX picture without rows or columns"))?;
        Ok(true)
    }
}

pub unsafe fn read_pcx_24bit(path: *const u8) -> Result<bool, BadInput> {
    unsafe {
        if !open_stream(stream(), path, 0)? {
            return Ok(false);
        }
        let rows = read_rows();
        let closed = close_stream(stream())?;
        Ok(rows? && closed)
    }
}
