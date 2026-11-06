//! `scene::upload_car_icon` (0x41B2F2): one car icon painted into a texture (the HUD's, kind 0xC1).
//! The game's log calls every texture a "icon"; this was `scene::upload_plate` until
//! 2026-10-24.
//!
//! In:  eax, ecx = the icon sheet's two pictures (the second for the alpha), ebx = the
//!      system texture, edx, esi = where on the sheet the icon is, edi, ebp = where on
//!      the texture it goes
//! Out: every register preserved
//!
//! - ebx, edi, ebp, edx, esi to `dword_5BE8F1`, `_8F5`, `_8F9`, `_8FD`, `_901`, the
//!   icon's size 0x52 x 0x2A to `dword_5BE905`, `_909`.
//! - With the texture's record (0x78 bytes at `unk_5B521C`) +0x5C at 1: the second
//!   picture read (`gfx::read_texture_picture`) and copied 0x40000 bytes on, or, if
//!   there is none or it cannot be read, +0x5C made 0.
//! - The first picture read; `least_colour` of the texture's format (record +0xC, +8);
//!   `alpha_from_kept` or `key_texture_colours` by +0x5C; `grade_texture`.
//! - The picture and the texture must be the same number of halvings from 256 (the
//!   table `dword_5ABA98`: 256, 128, 64, 32, and 1024 as 4), in width and in height;
//!   a size not in it, or two different numbers, is a fatal error (0x80A18761 ..
//!   0x80A18767). The picture is halved that many times (`halve_texture`) and the six
//!   places and sizes stored are shifted right by it.
//! - The texture locked (`sub_41AA4B`, which makes it `g_surface`); if it locked, the
//!   icon's piece blitted (`gfx::blit_components`, the picture's row bytes being its
//!   height times 4: the original's, a square sheet makes it right) and the texture
//!   unlocked (`sub_41AB02`). Those two are Direct3D and stay the game's.
//!
//! Two call sites in `scene::load_hud_car_icons`. Refuses a texture of 0x140 or more, a record
//! without its format, and what the pixel functions refuse; a refusal after the pictures are
//! read hands the original the call from the start.

use crate::errors::{BadInput, Refused};
use crate::gfx::blit_components::{Piece, blit_components, game_target};
use crate::gfx::read_texture_picture::read_texture_picture;
use crate::gfx::texture_pixels::{halvable, halve_texture};
use crate::gfx::texture_upload::{Record, halvings_apart, keep_second_picture, ready_picture};

// ---- game data ----

/// IDA's address of the upload's record.
pub const START: usize = 0x5BE8F1;
const STORED_LEN: usize = std::mem::size_of::<IconUpload>();

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct IconUpload {
    pub texture: u32,
    /// Where on the texture the icon goes.
    pub target_x: u32,
    pub target_y: u32,
    /// The icon's piece of the sheet: x, y, width, height (`blit_components`' rectangle).
    pub piece: [u32; 4],
}

const _: () = assert!(STORED_LEN == 0x1C);

/// `stru_5BE8F1` (`IconUpload` in IDA, unaligned): what the upload keeps while it works.
/// The record as bytes (it lies at an address that is no multiple of 4): `get` and `set` turn
/// them to and from an `IconUpload`.
#[repr(transparent)]
#[derive(rc2k_derive::Cells)]
pub struct IconUploadCell(crate::session::Bytes<STORED_LEN>);

impl IconUploadCell {
    pub fn get(&self) -> IconUpload {
        let bytes = self.0.read();
        let word =
            |n: usize| u32::from_le_bytes([bytes[n], bytes[n + 1], bytes[n + 2], bytes[n + 3]]);
        IconUpload {
            texture: word(0),
            target_x: word(4),
            target_y: word(8),
            piece: [word(12), word(16), word(20), word(24)],
        }
    }

    pub fn set(&self, upload: IconUpload) {
        let (texture, target_x, target_y, piece) = (
            upload.texture,
            upload.target_x,
            upload.target_y,
            upload.piece,
        );
        let mut bytes = [0u8; STORED_LEN];
        for (n, value) in [texture, target_x, target_y]
            .into_iter()
            .chain(piece)
            .enumerate()
        {
            bytes[4 * n..4 * n + 4].copy_from_slice(&value.to_le_bytes());
        }
        self.0.write(&bytes);
    }

    #[cfg(test)]
    pub fn addr(&self) -> u32 {
        crate::pointers::GamePtr::to(self).addr()
    }
}

impl IconUploadCell {
    pub const fn new() -> Self {
        IconUploadCell(crate::session::Bytes::new([0; STORED_LEN]))
    }
}

/// The piece's place: what `blit_components` gets in ebx, bounded to the piece's four dwords
/// (the longest read is its 16 bytes).
fn piece_place(upload: &IconUploadCell) -> crate::pointers::GamePtr {
    crate::pointers::GamePtr::to(upload)
        .offset(std::mem::offset_of!(IconUpload, piece) as u32)
        .limited(std::mem::size_of::<[u32; 4]>() as u32)
}

/// What `upload_car_icon` is given (the original's registers): the icon sheet's two pictures
/// (eax, ecx), the system texture (ebx), where on the sheet the icon is (edx, esi) and
/// where on the texture it goes (edi, ebp).
pub struct CarIcon {
    pub sheet: crate::pointers::GamePtr,
    pub alpha_sheet: crate::pointers::GamePtr,
    pub texture: u32,
    pub sheet_x: u32,
    pub sheet_y: u32,
    pub target_x: u32,
    pub target_y: u32,
}

// ---- Rust version ----

/// The Rust version. The original keeps every register (`pusha` .. `popa`).
pub fn upload_car_icon(upload: &IconUploadCell, icon: &CarIcon) -> Result<(), Refused> {
    let texture = icon.texture;
    let record = Record::of(texture)?;
    upload.set(IconUpload {
        texture,
        target_x: icon.target_x,
        target_y: icon.target_y,
        piece: [icon.sheet_x, icon.sheet_y, 0x52, 0x2A],
    });
    keep_second_picture(&record, Some(icon.alpha_sheet))?;
    let (pixels, width, height, _) = read_texture_picture(icon.sheet)?;
    ready_picture(&record, pixels, width, height)?;
    let across = halvings_apart(
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
    if !halvable(width, height, across) {
        return Err(BadInput("a picture that halves to nothing").into());
    }
    let (_, height) = halve_texture(pixels, width, height, across)?;
    let mut stored: IconUpload = upload.get();
    let shift = across & 31;
    stored.target_x >>= shift;
    stored.target_y >>= shift;
    stored.piece = stored.piece.map(|value| value >> shift);
    upload.set(stored);
    // The texture locked (making it the surface): if it locked, the icon's piece
    // blitted and the texture unlocked.
    if !crate::d3d::lock_texture::lock_texture(texture)?.carry {
        let piece = Piece {
            rectangle: piece_place(upload),
            x: stored.target_x,
            y: stored.target_y,
            pixels,
            row_bytes: height.wrapping_mul(4),
        };
        let blitted = blit_components(
            &crate::gfx::owner::gfx().pixel.formats,
            &piece,
            &game_target()?,
        );
        crate::d3d::unlock_texture::unlock_texture(texture)?;
        blitted?;
    }
    Ok(())
}

#[cfg(test)]
mod block_tests {
    use super::*;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(
            &crate::scene::owner::SCENE.icon_upload,
            START,
            START + STORED_LEN,
        );
        assert_eq!(
            crate::moved::place(START),
            crate::scene::owner::SCENE.icon_upload.addr() as usize
        );
    }

    #[test]
    fn the_piece_place_holds_its_four_dwords_and_no_more() {
        assert!(
            piece_place(&crate::scene::owner::SCENE.icon_upload)
                .read::<[u32; 4]>("t")
                .is_ok()
        );
        assert!(
            piece_place(&crate::scene::owner::SCENE.icon_upload)
                .read::<[u32; 5]>("t")
                .is_err()
        );
    }
}
