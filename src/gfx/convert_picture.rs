//! The picture in the picture buffer made ready for the screen. A picture is 32-bit pixels of
//! four components, which is the screen's own format, so nothing is converted: what is left is
//! the check that there is a picture. Refuses no picture buffer.

use crate::errors::BadInput;
use crate::gfx::picture::has_picture;

/// The picture checked: it is already in the screen's format.
pub fn convert_picture_now_in(gfx: &crate::gfx::owner::Gfx) -> Result<(), BadInput> {
    if !has_picture(&gfx.pictures) {
        return Err(BadInput("no picture buffer"));
    }
    Ok(())
}
