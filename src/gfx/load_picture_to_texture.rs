//! `gfx::load_picture_to_texture` (0x405313): a picture file into a system texture.
//!
//! With the record's +0x5C at 1 the second picture is read and kept 0x40000 bytes on the
//! picture's start, or +0x5C made 0 if there is none or it cannot be read
//! (`gfx/texture_upload.rs`); then the picture is read (`gfx::read_texture_picture`, 256 x 256
//! of -1 if it cannot be) and `gfx::upload_texture_picture` puts it in. Three call sites in
//! `scene::load_texture_picture`, which calls this. Refuses a texture of 0x140
//! or more, a record without its format, and what its parts refuse.

use crate::errors::Refused;
use crate::gfx::read_texture_picture::read_texture_picture;
use crate::gfx::texture_upload::{Record, keep_second_picture};
use crate::gfx::upload_texture_picture::{GameSurfaces, upload_texture_picture};
use crate::session::TextPlace;

/// `upload` gets the texture and what the reading left: the pixels, the width and the
/// height.
pub fn load_picture_to_texture(
    path: Option<&dyn TextPlace>,
    second: Option<&dyn TextPlace>,
    texture: u32,
    upload: &mut dyn FnMut(u32, u32, u32, u32) -> Result<(), Refused>,
) -> Result<(), Refused> {
    let record = Record::of(texture)?;
    keep_second_picture(&record, second)?;
    let (pixels, width, height, _) = read_texture_picture(path)?;
    upload(texture, pixels, width, height)
}

/// With the Rust upload into the game's textures.
pub fn load_picture_to_texture_in_game(
    path: Option<&dyn TextPlace>,
    second: Option<&dyn TextPlace>,
    texture: u32,
    gfx: &crate::gfx::owner::Gfx,
) -> Result<(), Refused> {
    load_picture_to_texture(
        path,
        second,
        texture,
        &mut |texture, pixels, width, height| {
            upload_texture_picture(texture, pixels, width, height, &mut GameSurfaces(gfx), gfx)
        },
    )
}

// gfx::upload_texture_picture
