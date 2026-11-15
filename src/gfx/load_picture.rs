//! Loads a picture file into the picture buffer at the screen's size. The path may be written to
//! (see `files::swap_missing_extension`).
//! Sets the picture's size to 800 x 600 with rows that follow one another, clears the buffer and
//! calls `read_picture_file`. Whatever that says, the picture's size then becomes the screen's and
//! its row length four bytes a pixel. If the file was read and its decoded width is not the
//! screen's, `rescale_picture` makes it so. Refuses what the functions it calls refuse.

use crate::errors::Refused;
use crate::gfx::clear_picture::clear_picture;
use crate::gfx::picture::has_picture;
use crate::gfx::picture::set_size;
use crate::gfx::read_picture_file::read_picture_file;
use crate::gfx::rescale_picture::rescale_picture;
use rc2k_core::gfx::picture_size as sizes;

/// Returns whether the file was read.
pub fn load_picture_with(
    path: &dyn crate::memory::session::TextPlace,
    gfx: &crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
) -> Result<bool, Refused> {
    // Refused before anything is written, if there is no buffer.
    let pictures = &gfx.pictures;
    if !has_picture(pictures) {
        clear_picture(pictures)?;
    }
    set_size(pictures, sizes::READ);
    clear_picture(pictures)?;
    let read = read_picture_file(path, gfx, system);
    let screen_width: u32 = system.session.screen.get().width.get();
    set_size(
        pictures,
        sizes::screen(screen_width, system.session.screen.get().height.get()),
    );
    if !read? {
        return Ok(false);
    }
    if sizes::rescaled(gfx.pictures.decoder.width.get(), screen_width) {
        rescale_picture(&gfx.rescale, gfx)?;
    }
    Ok(true)
}
