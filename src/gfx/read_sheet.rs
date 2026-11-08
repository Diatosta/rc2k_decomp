//! Two of the game's picture readers as Rust functions, for the sprite and font builder.
//! `read_sheet` is `gfx::read_sheet` (0x4119B7): it zeroes the PCX header, reads the file's first
//! 0x80 bytes and goes by the planes: one to `read_pcx_indexed`, three to
//! `gfx::read_pcx_24bit`, anything else fails. `read_pcx_indexed` is `gfx::read_pcx_indexed`
//! (0x411B93):
//! `gfx::read_pcx_8bit` but for two things: `g_pictureFlags` becomes 0x100, so that the layout
//! is of one byte a pixel, and what is stored is the palette number itself.

use crate::errors::BadInput;
use crate::files::read_stream_byte::read_stream_byte;
use crate::files::stream::Stream;
use crate::gfx::lay_out_pcx::lay_out_pcx;
use crate::gfx::pcx::{HEADER_PLANES, on_stream, read_header, read_stream_header};
use crate::gfx::picture::{put_or_end, with_buffer};
use crate::gfx::read_pcx_8bit::read_file_palette;
use crate::gfx::read_pcx_24bit::read_pcx_24bit;

/// The part between the open and the close: whether it worked.
fn read_rows(stream: &mut Stream) -> Result<bool, BadInput> {
    let gfx = crate::gfx::owner::gfx();
    let pictures = &gfx.pictures;
    if !read_file_palette(stream, gfx) || !read_stream_header(stream, gfx)? {
        return Ok(false);
    }
    pictures.picture.flags.set(0x100u32);
    lay_out_pcx(gfx)?;

    let (width, height, line_bytes): (u32, u32, u32) = (
        pictures.decoder.width.get(),
        pictures.decoder.height.get(),
        pictures.pcx.line_bytes.get(),
    );
    if height == 0 {
        return Err(BadInput("a PCX picture without rows"));
    }
    let gap: u32 = pictures.decoder.row_gap.get();
    // The byte offset in the decoded buffer.
    let mut to = 0usize;
    // The byte a run repeats, and how much of the run is left.
    let (mut byte, mut run) = (0u8, 0u32);
    with_buffer(
        pictures.decoder.pixels.get(),
        |pixels| -> Result<(), BadInput> {
            for _ in 0..height {
                let mut column = 0u32;
                loop {
                    if run == 0 {
                        byte = read_stream_byte(stream)?.value();
                        run = 1;
                        if byte >= 0xC0 {
                            run = byte as u32 - 0xC0;
                            byte = read_stream_byte(stream)?.value();
                        }
                    }
                    if column < width {
                        put_or_end(pixels, to, &[byte]);
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
        },
    )??;
    Ok(true)
}

/// Returns whether the picture was read.
pub fn read_pcx_indexed(path: &[u8]) -> Result<bool, BadInput> {
    on_stream(path, read_rows)
}

/// Returns whether the picture was read.
pub fn read_sheet(path: &[u8], gfx: &crate::gfx::owner::Gfx) -> Result<bool, BadInput> {
    if !read_header(path, gfx)? {
        return Ok(false);
    }
    match gfx.pictures.pcx.header.get(HEADER_PLANES) {
        1 => read_pcx_indexed(path),
        3 => read_pcx_24bit(path),
        _ => Ok(false),
    }
}
