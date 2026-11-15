//! Reads a texture's picture into the picture buffer: the picture's size 256 x 256 with rows that
//! follow one another, its first 0x10000 dwords filled with -1, then `read_picture_file` if there
//! is a path. Refuses no picture buffer, and what `read_picture_file` refuses.

use crate::errors::{BadInput, Refused};
use crate::gfx::read_picture_file::read_picture_file;
use crate::memory::session::TextPlace;
use rc2k_core::gfx::picture_size as sizes;

/// Returns the pixels (a picture buffer's number, `gfx::picture`), their width and height, and
/// whether the reading failed. `path` is `None` for no picture to read.
pub fn read_texture_picture(
    path: Option<&dyn TextPlace>,
    gfx: &crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
) -> Result<(u32, u32, u32, bool), Refused> {
    let pictures = &gfx.pictures;
    let buffer = pictures.picture.buffer.get();
    if !crate::gfx::picture::has_picture(pictures) {
        return Err(BadInput("no picture buffer").into());
    }
    crate::gfx::picture::set_size(pictures, sizes::TEXTURE);
    crate::gfx::picture::with_picture(pictures, |b| {
        b.get_mut(..sizes::TEXTURE_FILLED * 4).map(|p| p.fill(0xFF))
    })?
    .ok_or(BadInput("a picture buffer shorter than a texture"))?;
    let failed = match path {
        None => false,
        Some(path) if read_picture_file(path, gfx, system)? => {
            return Ok((
                pictures.decoder.pixels.get(),
                pictures.decoder.width.get(),
                pictures.decoder.height.get(),
                false,
            ));
        }
        Some(_) => true,
    };
    Ok((
        buffer,
        pictures.picture.width.get(),
        pictures.picture.height.get(),
        failed,
    ))
}
