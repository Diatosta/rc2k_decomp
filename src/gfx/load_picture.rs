//! `gfx::load_picture` (0x411863): loads a picture file into the picture buffer at the
//! screen's size.
//!
//! Takes: path, which may be written to (see `files::swap_missing_extension`)
//! Sets the picture's size to 800 x 600 with rows that follow one another (row length
//! 0xFFFFFFFD), clears the buffer and calls `read_picture_file`. Whatever that says, the
//! picture's size then becomes the screen's (`g_screenWidth` x `g_screenHeight`) and its row
//! length four bytes a pixel. If the file was read and its decoded width is not the
//! screen's, `rescale_picture` makes it so.
//! Ten callers, `menu::load_background` and `service::load_background` among them.
//! Refuses what the functions it calls refuse. The name is a reading.

use crate::errors::Refused;
use crate::gfx::clear_picture::clear_picture;
use crate::gfx::owner::gfx;
use crate::gfx::picture::has_picture;
use crate::gfx::picture::set_size;
use crate::gfx::read_picture_file::read_picture_file;
use crate::gfx::rescale_picture::rescale_picture;
use crate::system::owner::system;
use rc2k_core::gfx::picture_size as sizes;

// ---- game data ----

/// Returns whether the file was read.
pub fn load_picture(path: &dyn crate::session::TextPlace) -> Result<bool, Refused> {
    let system = system();
    let gfx = gfx();
    // Refused before anything is written, if there is no buffer.
    if !has_picture() {
        clear_picture()?;
    }
    // Converted: `rc2k_core::gfx::picture_size` (`READ`, `screen`, `rescaled`).
    set_size(sizes::READ);
    clear_picture()?;
    let read = read_picture_file(path, gfx);
    let screen_width: u32 = system.session.screen.width.get();
    set_size(sizes::screen(
        screen_width,
        system.session.screen.height.get(),
    ));
    if !read? {
        return Ok(false);
    }
    if sizes::rescaled(gfx.pictures.decoder.width.get(), screen_width) {
        rescale_picture(&gfx.rescale, gfx)?;
    }
    Ok(true)
}
