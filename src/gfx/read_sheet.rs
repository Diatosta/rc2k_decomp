//! Two of the game's picture readers as Rust functions, for the sprite and font builder.
//! `read_sheet` is `sub_4119B7` (0x4119B7): it zeroes the PCX header, reads the file's first
//! 0x80 bytes and goes by the planes: one to `read_pcx_indexed`, three to
//! `gfx::read_pcx_24bit`, anything else fails. `read_pcx_indexed` is `sub_411B93` (0x411B93):
//! `gfx::read_pcx_8bit` but for two things: `g_pictureFlags` becomes 0x100, so that the layout
//! is of one byte a pixel, and what is stored is the palette number itself.

use crate::errors::BadInput;
use crate::files::close_stream::close_stream;
use crate::files::handle::{read, seek};
use crate::files::open_stream::open_stream;
use crate::files::read_file::read_file;
use crate::files::read_stream_block::read_stream_block;
use crate::files::read_stream_byte::read_stream_byte;
use crate::gfx::decoder::DECODER;
use crate::gfx::lay_out_pcx::lay_out_pcx;
use crate::gfx::pcx::PCX;
use crate::gfx::pcx::{FILE_PALETTE_LEN, HEADER_LEN, HEADER_PLANES, stream};
use crate::gfx::picture::{put_or_end, with_buffer};
use crate::gfx::picture_store::PICTURE;
use crate::gfx::read_pcx_24bit::read_pcx_24bit;
use crate::system::zero_memory::zero_memory;

/// The part between the open and the close: whether it worked.
unsafe fn read_rows() -> Result<bool, BadInput> {
    unsafe {
        let stream = stream();
        let file = (*stream).file;
        if seek(file, -(FILE_PALETTE_LEN as i32), 2).is_none() {
            return Ok(false);
        }
        if !read(
            file,
            crate::pointers::host(PCX.file_palette.addr()),
            FILE_PALETTE_LEN as u32,
        )
        .1
        {
            return Ok(false);
        }
        if seek(file, 0, 0).is_none() {
            return Ok(false);
        }
        if read_stream_block(
            stream,
            crate::pointers::host(PCX.header.addr()),
            HEADER_LEN as u32,
        )?
        .1
        {
            return Ok(false);
        }
        PICTURE.flags.set(0x100u32);
        lay_out_pcx()?;

        let (width, height, line_bytes): (u32, u32, u32) = (
            DECODER.width.get(),
            DECODER.height.get(),
            PCX.line_bytes.get(),
        );
        if height == 0 {
            return Err(BadInput("a PCX picture without rows"));
        }
        let gap: u32 = DECODER.row_gap.get();
        // The byte offset in the decoded buffer.
        let mut to = 0usize;
        // eax as the original keeps it (al is the byte a run repeats), and what is left of
        // the run.
        let (mut eax, mut run) = (0u32, 0u32);
        with_buffer(DECODER.pixels.get(), |pixels| -> Result<(), BadInput> {
            for _ in 0..height {
                let mut column = 0u32;
                loop {
                    if run == 0 {
                        eax = read_stream_byte(stream, eax)?.eax;
                        run = 1;
                        if eax as u8 >= 0xC0 {
                            run = eax as u8 as u32 - 0xC0;
                            eax = read_stream_byte(stream, eax)?.eax;
                        }
                    }
                    if column < width {
                        put_or_end(pixels, to, &[eax as u8]);
                        to = to.wrapping_add(1);
                    }
                    run = run.wrapping_sub(1);
                    column = column.wrapping_add(1);
                    if column >= line_bytes {
                        break;
                    }
                }
                to = to.wrapping_add(gap as usize);
            }
            Ok(())
        })??;
        Ok(true)
    }
}

/// Returns whether the picture was read.
pub unsafe fn read_pcx_indexed(path: *const u8) -> Result<bool, BadInput> {
    unsafe {
        if !open_stream(stream(), path, 0)? {
            return Ok(false);
        }
        let rows = read_rows();
        let closed = close_stream(stream())?;
        Ok(rows? && closed)
    }
}

/// Returns whether the picture was read.
pub unsafe fn read_sheet(path: *const u8) -> Result<bool, BadInput> {
    unsafe {
        let header = crate::pointers::host(PCX.header.addr());
        zero_memory(header, HEADER_LEN as u32)?;
        if !read_file(path, header, HEADER_LEN as u32, 0)?.ok {
            return Ok(false);
        }
        match PCX.header.get(HEADER_PLANES) {
            1 => read_pcx_indexed(path),
            3 => read_pcx_24bit(path),
            _ => Ok(false),
        }
    }
}
