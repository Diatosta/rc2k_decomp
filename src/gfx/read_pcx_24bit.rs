//! Decodes a PCX file of three planes, 8 bits each, into the picture buffer as 32-bit pixels.
//! Opens the file and reads all of it at once; the header is its first 0x80 bytes (what a shorter
//! file lacks stays as it was); sets the picture flags to 0x800 and calls `lay_out_pcx`. Then the
//! rows. A row in the file is one plane after the other, each the header's count of bytes; plane
//! n's bytes go to byte `planes - 1 - n` of the pixels (so the first plane is the third byte),
//! those within the decoded width. Then the fourth byte of every pixel of the row is set to 0, and
//! the destination moves on by the stride. A run goes on across the ends of planes and rows, but
//! the byte it repeats is 0 from the end of a row on. A file that ends early is filled up with 0.
//! It fails if the open or the close fails, or a read before the header is whole. Refuses a
//! decoded width or height of 0 and what the open refuses; a pixel past the picture buffer ends
//! the game.

use crate::errors::BadInput;
use crate::files::handle::{close, handle_size, open_by_mode, read_slice};
use crate::gfx::lay_out_pcx::lay_out_pcx;
use crate::gfx::pcx::HEADER_LEN;
use crate::gfx::picture::with_buffer;

/// All of an open file's bytes, read at once (again while more comes, should the file be longer
/// than it said), and whether no read failed.
fn read_whole(file: u32) -> (Vec<u8>, bool) {
    let mut bytes = Vec::new();
    // One byte more than the size: a read that fills it all says there may be more.
    let mut step = handle_size(file).map_or(0x2000, |size| size as usize + 1);
    loop {
        let had = bytes.len();
        bytes.resize(had + step, 0);
        let (got, ok) = read_slice(file, &mut bytes[had..]);
        bytes.truncate(had + got as usize);
        if !ok || (got as usize) < step {
            return (bytes, ok);
        }
        step = 0x2000;
    }
}

pub fn read_pcx_24bit(path: &[u8], gfx: &crate::gfx::owner::Gfx) -> Result<bool, BadInput> {
    let Some(file) = open_by_mode(path, 0)? else {
        return Ok(false);
    };
    let (bytes, read) = read_whole(file);
    let closed = close(file);
    let pictures = &gfx.pictures;
    let (head, rows) = bytes.split_at(bytes.len().min(HEADER_LEN));
    let mut header = pictures.pcx.header.read();
    header[..head.len()].copy_from_slice(head);
    pictures.pcx.header.write(&header);
    if head.len() < HEADER_LEN && !read {
        return Ok(false);
    }
    pictures.picture.flags.set(0x800u32);
    let layout = lay_out_pcx(gfx)?;
    let decoded = with_buffer(&pictures.buffers, pictures.decoder.pixels.get(), |pixels| {
        rc2k_core::gfx::pcx::decode_planes_in(&layout, rows, pixels)
    })?;
    match decoded {
        Ok(Some(())) => Ok(closed),
        Ok(None) => Err(BadInput("a PCX picture without rows or columns")),
        Err(past) => crate::fatal::raise::end("gfx::picture", BadInput::from(past)),
    }
}
