//! A picture loaded, and a second laid over it but for its magenta: `gfx::load_picture` of the
//! first; if it was read, the overlay is read into memory of its own (Rust's,
//! `gfx::picture::Buffer::Overlay`) while the picture's size, row length and buffer are kept, then
//! put back; each overlay pixel that is not magenta goes over the picture. The work list is given
//! back. Refuses an overlay read with no width or height, and what its parts refuse.

use crate::errors::{BadInput, Refused};
use crate::gfx::load_picture::load_picture_with;
use crate::gfx::picture::{Buffer, take, with_buffer};
use crate::gfx::read_picture_file::read_picture_file;
use crate::memory::free_list::free_list;

const MEMORY: u32 = 0x1D4C00 + 0xC80;

/// Returns whether the picture could not be read.
pub fn load_picture_with_overlay_in(
    path: &dyn crate::session::TextPlace,
    overlay_path: &dyn crate::session::TextPlace,
    gfx: &crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
    strings: &crate::strings::owner::Strings,
) -> Result<bool, Refused> {
    let pictures = &gfx.pictures;
    if !load_picture_with(path, gfx, system)? {
        return Ok(true);
    }
    let list = &strings.lexicon.memory.memory;
    // The overlay's buffer is Rust's (`gfx::picture`), counted on the list as the game's block was,
    // given back with it.
    let free = || -> Result<(), Refused> {
        crate::gfx::picture::give_back(Buffer::Overlay);
        Ok(free_list(list)?)
    };
    let overlay = take(Buffer::Overlay, list, MEMORY)?;
    let kept: [u32; 4] = [
        pictures.picture.width.get(),
        pictures.picture.height.get(),
        pictures.picture.row_bytes.get(),
        pictures.picture.buffer.get(),
    ];
    pictures.picture.buffer.set(overlay);
    let read = read_picture_file(overlay_path, gfx, system);
    pictures.picture.width.set(kept[0]);
    pictures.picture.height.set(kept[1]);
    pictures.picture.row_bytes.set(kept[2]);
    pictures.picture.buffer.set(kept[3]);
    let read = match read {
        Ok(read) => read,
        Err(e) => {
            free()?;
            return Err(e);
        }
    };
    if read {
        let (width, height): (u32, u32) =
            (pictures.decoder.width.get(), pictures.decoder.height.get());
        let (stride, row): (u32, u32) = (
            pictures.decoder.stride.get(),
            pictures.picture.row_bytes.get(),
        );
        let laid = with_buffer(overlay, |from| {
            with_buffer(kept[3], |to| {
                match (to.get_mut(..MEMORY as usize), from.get(..MEMORY as usize)) {
                    (Some(to), Some(from)) => rc2k_core::gfx::loading::lay_over(
                        to,
                        from,
                        width as usize,
                        height as usize,
                        stride as usize,
                        row as usize,
                    )
                    .map_err(|e| BadInput(e.0)),
                    _ => Err(BadInput("a picture buffer shorter than its room")),
                }
            })
        })
        .and_then(|r| r)
        .and_then(|r| r);
        if let Err(e) = laid {
            free()?;
            return Err(e.into());
        }
    }
    free()?;
    Ok(false)
}
