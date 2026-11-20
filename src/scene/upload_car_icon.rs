//! `scene::upload_car_icon`: one car icon painted into a texture (the HUD's, kind 0xC1). The
//! game's log calls every texture a "plate".
//!
//! - The texture, the icon's place on the texture and on the sheet are held in an
//!   `IconUpload`, and the icon's size 0x52 x 0x2A with them.
//! - With the texture's record (0x78 bytes) `keeps_alpha` at 1: the second picture read
//!   (`gfx::read_texture_picture`) and copied 0x40000 bytes on, or, if there is none or it
//!   cannot be read, `keeps_alpha` made 0.
//! - The first picture read; `least_colour` of the texture's format (the record's `format`);
//!   `alpha_from_kept` or `key_texture_colours` by `keeps_alpha`; `grade_texture`.
//! - The picture and the texture must be the same number of halvings from 256 (the card's
//!   sizes table: 256, 128, 64, 32, and 1024 as 4), in width and in height; a size not in it,
//!   or two different numbers, is a fatal error (0x80A18761 .. 0x80A18767). The picture is
//!   halved that many times (`halve_texture`) and the six places and sizes stored are shifted
//!   right by it.
//! - The texture locked (`d3d::lock_texture`, which makes it the blitters' surface); if it
//!   locked, the icon's piece blitted (`gfx::blit_components`, the picture's row bytes being
//!   its height times 4: right for a square sheet) and the texture unlocked
//!   (`d3d::unlock_texture`).
//!
//! Two call sites in `scene::load_hud_car_icons`. Refuses a texture of 0x140 or more, a record
//! without its format, and what the pixel functions refuse; a refusal after the pictures are
//! read leaves the call to be made again from the start.

use crate::errors::{BadInput, Refused};
use crate::gfx::blit_components::{Piece, blit_components, game_target};
use crate::gfx::read_texture_picture::read_texture_picture;
use crate::gfx::texture_pixels::{halvable, halve_texture};
use crate::gfx::texture_upload::{Record, halvings_apart, keep_second_picture, ready_picture};
use crate::scene::icon_upload_store::IconUpload;

/// What `upload_car_icon` is given: the icon sheet's two pictures, the system texture, where on
/// the sheet the icon is and where on the texture it goes.
pub struct CarIcon<'a> {
    pub sheet: crate::scene::file_numbers_store::PathText<'a>,
    pub alpha_sheet: crate::scene::file_numbers_store::PathText<'a>,
    pub texture: u32,
    pub sheet_x: u32,
    pub sheet_y: u32,
    pub target_x: u32,
    pub target_y: u32,
    /// The icon painted at half its size (its piece and place are the full size's).
    pub half: bool,
}

/// The upload.
pub fn upload_car_icon(
    icon: &CarIcon<'_>,
    gfx: &crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    let texture = icon.texture;
    let record = Record::of(texture, gfx)?;
    let mut stored = IconUpload {
        target_x: icon.target_x,
        target_y: icon.target_y,
        piece: [icon.sheet_x, icon.sheet_y, 0x52, 0x2A],
    };
    keep_second_picture(&record, Some(&icon.alpha_sheet), gfx, system)?;
    let (pixels, width, height, _) = read_texture_picture(Some(&icon.sheet), gfx, system)?;
    ready_picture(&record, pixels, width, height, gfx)?;
    let across = halvings_apart(
        gfx,
        &record,
        width,
        height,
        [
            0x80A1_8761,
            0x80A1_8762,
            0x80A1_8764,
            0x80A1_8765,
            0x80A1_8767,
        ],
    )?;
    // A half-size icon: the picture is halved once more and the piece taken from it.
    let across = if icon.half { across + 1 } else { across };
    if !halvable(width, height, across) {
        return Err(BadInput("a picture that halves to nothing").into());
    }
    let (_, height) = halve_texture(&gfx.pictures.buffers, pixels, width, height, across)?;
    let shift = across & 31;
    // The place on the texture is in the texture's own pixels: the halvings the picture is from
    // the texture, not the extra one.
    let place_shift = if icon.half { shift - 1 } else { shift };
    stored.target_x >>= place_shift;
    stored.target_y >>= place_shift;
    stored.piece = stored.piece.map(|value| value >> shift);
    // The texture locked (making it the surface): if it locked, the icon's piece
    // blitted and the texture unlocked.
    if crate::d3d::lock_texture::lock_texture_in(gfx, texture)? {
        let piece = Piece {
            rectangle: stored.piece,
            x: stored.target_x,
            y: stored.target_y,
            pixels,
            row_bytes: height.wrapping_mul(4),
        };
        let blitted = blit_components(&gfx.pixel.formats, &piece, &game_target(gfx)?, gfx, system);
        crate::d3d::unlock_texture::unlock_texture(gfx, texture)?;
        blitted?;
    }
    Ok(())
}
