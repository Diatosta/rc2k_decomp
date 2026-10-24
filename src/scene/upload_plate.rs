//! `scene::upload_plate` (0x41B2F2): one number plate painted into a texture.
//!
//! In:  eax, ecx = the plate sheet's two pictures (the second for the alpha), ebx = the
//!      system texture, edx, esi = where on the sheet the plate is, edi, ebp = where on
//!      the texture it goes
//! Out: every register preserved
//!
//! - ebx, edi, ebp, edx, esi to `dword_5BE8F1`, `_8F5`, `_8F9`, `_8FD`, `_901`, the
//!   plate's size 0x52 x 0x2A to `dword_5BE905`, `_909`.
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
//!   plate's piece blitted (`gfx::blit_components`, the picture's row bytes being its
//!   height times 4: the original's, a square sheet makes it right) and the texture
//!   unlocked (`sub_41AB02`). Those two are Direct3D and stay the game's.
//!
//! Two call sites in `scene::load_number_plates`.
//! Refuses a texture of 0x140 or more, a record without its format, and what the
//! pixel functions refuse; a refusal after the pictures are read hands the original
//! the call from the start.
//! Shadow mode runs the original alone.
//! Self-test: a redirected copy whose lock, blit and unlock note what they are given,
//! on sheets of the test's in a texture record of the test's.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{game_addr, game_read, game_write, rva};
use crate::gfx::blit_components::{Piece, blit_components, game_target};
use crate::gfx::read_texture_picture::read_texture_picture;
use crate::gfx::texture_pixels::{halvable, halve_texture};
use crate::gfx::texture_upload::{Record, halvings_apart, keep_second_picture, ready_picture};

// ---- game data ----

/// `stru_5BE8F1` (`PlateUpload` in IDA, unaligned): what the upload keeps while it works.
const TEXTURE_RVA: usize = rva(0x5BE8F1);
const STORED_LEN: usize = std::mem::size_of::<PlateUpload>();

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct PlateUpload {
    pub texture: u32,
    /// Where on the texture the plate goes.
    pub target_x: u32,
    pub target_y: u32,
    /// The plate's piece of the sheet: x, y, width, height (`blit_components`' rectangle).
    pub piece: [u32; 4],
}

const _: () = assert!(STORED_LEN == 0x1C);

/// The piece's place: what `blit_components` gets in ebx.
fn piece_address() -> u32 {
    (game_addr(TEXTURE_RVA) + std::mem::offset_of!(PlateUpload, piece)) as u32
}

/// What `upload_plate` is given (the original's registers): the plate sheet's two pictures
/// (eax, ecx), the system texture (ebx), where on the sheet the plate is (edx, esi) and
/// where on the texture it goes (edi, ebp).
pub struct Plate {
    pub sheet: u32,
    pub alpha_sheet: u32,
    pub texture: u32,
    pub sheet_x: u32,
    pub sheet_y: u32,
    pub target_x: u32,
    pub target_y: u32,
}

// ---- Rust version ----

/// The Rust version. The original keeps every register (`pusha` .. `popa`).
pub unsafe fn upload_plate(plate: &Plate) -> Result<(), Refused> {
    unsafe {
        let texture = plate.texture;
        let record = Record::of(texture)?;
        game_write(
            TEXTURE_RVA,
            PlateUpload {
                texture,
                target_x: plate.target_x,
                target_y: plate.target_y,
                piece: [plate.sheet_x, plate.sheet_y, 0x52, 0x2A],
            },
        );
        keep_second_picture(&record, plate.alpha_sheet)?;
        let (pixels, width, height, _) = read_texture_picture(plate.sheet as *mut u8)?;
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
        let (_, height) = halve_texture(pixels as *mut u32, width, height, across);
        let mut stored: PlateUpload = game_read(TEXTURE_RVA);
        let shift = across & 31;
        stored.target_x >>= shift;
        stored.target_y >>= shift;
        stored.piece = stored.piece.map(|value| value >> shift);
        game_write(TEXTURE_RVA, stored);
        // The texture locked (making it the surface): if it locked, the plate's piece
        // blitted and the texture unlocked.
        if !crate::d3d::lock_texture::lock_texture(texture)?.carry {
            let piece = Piece {
                rectangle: piece_address(),
                x: stored.target_x,
                y: stored.target_y,
                pixels,
                row_bytes: height.wrapping_mul(4),
            };
            let blitted = blit_components(&piece, &game_target());
            crate::d3d::unlock_texture::unlock_texture(texture)?;
            blitted?;
        }
        Ok(())
    }
}
