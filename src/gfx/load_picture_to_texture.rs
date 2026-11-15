//! A picture file for a system texture: read (`read_picture_for_texture`), then put into every
//! level of the texture (`upload_picture`).
//!
//! With the texture's `keeps_alpha` at 1 the second picture is read first and kept 0x40000 bytes
//! on the picture's start, or `keeps_alpha` is made 0 if there is none or it cannot be read
//! (`gfx/texture_upload.rs`). The picture is 256 x 256 of -1 if it cannot be read. Refuses a
//! texture of 0x140 or more, a record without its format, and what the parts refuse.

use crate::errors::Refused;
use crate::gfx::read_texture_picture::read_texture_picture;
use crate::gfx::texture_upload::{Record, keep_second_picture};
use crate::gfx::upload_texture_picture::{GameSurfaces, upload_texture_picture};
use crate::session::TextPlace;

/// A texture's picture, read: the picture buffer's handle and the picture's size.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TexturePicture {
    pub texture: u32,
    pub pixels: u32,
    pub width: u32,
    pub height: u32,
}

/// The picture at `path` read for `texture` (`second` is the alpha picture).
pub fn read_picture_for_texture(
    path: Option<&dyn TextPlace>,
    second: Option<&dyn TextPlace>,
    texture: u32,
    gfx: &crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
) -> Result<TexturePicture, Refused> {
    let record = Record::of(texture, gfx)?;
    keep_second_picture(&record, second, gfx, system)?;
    let (pixels, width, height, _) = read_texture_picture(path, gfx, system)?;
    Ok(TexturePicture {
        texture,
        pixels,
        width,
        height,
    })
}

/// The picture put into the game's texture.
pub fn upload_picture(
    picture: &TexturePicture,
    gfx: &crate::gfx::owner::Gfx,
) -> Result<(), Refused> {
    upload_texture_picture(
        picture.texture,
        picture.pixels,
        picture.width,
        picture.height,
        &mut GameSurfaces(gfx),
        gfx,
    )
}
