//! `gfx::upload_texture_picture` (0x41A68B): a picture into every level of a system
//! texture.
//!
//! - The picture made ready (`gfx/texture_upload.rs`: the least colour, the alpha from
//!   the kept picture or the colours keyed, the grading).
//! - Picture and texture the same number of halvings from 256 across and down, else a
//!   fatal error (0x80A18741 .. 0x80A18747).
//! - For each of the record's levels (+0x10; one at least): the picture halved (that
//!   many times for the first, once for each after), the level's surface (+0x14 ..)
//!   locked through its `Lock` (DirectDraw, vtable +0x64, into the description at
//!   `dword_5ABA18`), every pixel through `components_to_pixel` with the texture's
//!   format into it as words or dwords by the kind's bytes a pixel (another number: a
//!   fatal error 0x80A18775), rows `dword_5ABA28` apart from `dword_5ABA3C`, and
//!   `Unlock` (+0x80).
//! - A `Lock` or `Unlock` that fails is a fatal error (0x80218772, 0x80218773) when
//!   `dword_4F52AC` is 1; otherwise the original returns with three dwords still
//!   pushed, so its `popa` and `ret` go astray: refused.
//!
//! One caller, `gfx::load_picture_to_texture` (the other reference, 0x425A52, is in code
//! nothing reaches). The two DirectDraw calls go through `Surfaces`, which the standalone
//! program can answer with textures of its own. Refuses a texture of 0x140 or more, a record
//! without its format, more than nine levels, a picture that halves to nothing, and what
//! `ready_picture` refuses; a refusal after the picture was changed hands the original the
//! changed picture.

use crate::errors::{BadInput, Refused};
use crate::gfx::components_to_pixel::components_to_pixel;
use crate::gfx::owner::gfx;
use crate::gfx::texture_pixels::{halvable, halve_texture};
use crate::gfx::texture_upload::{Record, fatal, halvings_apart, ready_picture};
use rc2k_core::gfx::uploads::{self};

// ---- game data ----

/// A level's surface locked and unlocked; an error is what DirectDraw returned.
pub trait Surfaces {
    /// The pixels and the pitch.
    fn lock(&mut self, surface: u32) -> Result<(u32, u32), i32>;
    fn unlock(&mut self, surface: u32) -> Result<(), i32>;
}

/// The game's: DirectDraw's own calls, the description being the game's.
pub struct GameSurfaces;

impl Surfaces for GameSurfaces {
    fn lock(&mut self, surface: u32) -> Result<(u32, u32), i32> {
        let card = &gfx().card;
        match crate::platform_3d::lock_surface(surface, &card.upload.description) {
            0 => Ok((
                card.upload.description.pixels.get(),
                card.upload.description.pitch.get(),
            )),
            e => Err(e),
        }
    }

    fn unlock(&mut self, surface: u32) -> Result<(), i32> {
        match crate::platform_3d::unlock_surface(surface) {
            0 => Ok(()),
            e => Err(e),
        }
    }
}

fn direct_draw_failed(code: u32, gfx: &crate::gfx::owner::Gfx) -> Refused {
    if gfx.reports.on.get() == 1 {
        fatal(code)
    } else {
        BadInput("DirectDraw failed and the original's stack goes astray").into()
    }
}

pub fn upload_texture_picture(
    texture: u32,
    pixels: u32,
    width: u32,
    height: u32,
    surfaces: &mut dyn Surfaces,
    gfx: &crate::gfx::owner::Gfx,
) -> Result<(), Refused> {
    if pixels == 0 {
        return Ok(());
    }
    // Converted: `rc2k_core::gfx::uploads` (`levels`, `halvings_before`; the halvings
    // through `texture_upload::halvings_apart`).
    let record = Record::of(texture)?;
    let levels = uploads::levels(record.get().levels).map_err(BadInput::from)?;
    ready_picture(&record, pixels, width, height, gfx)?;
    let apart = halvings_apart(
        &record,
        width,
        height,
        [
            0x80A1_8741,
            0x80A1_8742,
            0x80A1_8744,
            0x80A1_8745,
            0x80A1_8747,
        ],
    )?;
    if !halvable(width, height, apart + levels - 1) {
        return Err(BadInput("a picture that halves to nothing").into());
    }
    let format = record.format()?;
    let bytes = record.bytes_per_pixel()?;
    let (mut width, mut height) = (width, height);
    for level in 0..levels {
        let times = uploads::halvings_before(level, apart);
        if times != 0 {
            (width, height) = halve_texture(pixels, width, height, times)?;
        }
        let surface = record.get().surfaces[level as usize];
        let (target, pitch) = surfaces
            .lock(surface)
            .map_err(|_| direct_draw_failed(0x8021_8772, gfx))?;
        // The level's pixels read once; the surface is the platform's.
        let picture =
            crate::gfx::picture::read_bytes(pixels, 0, width as usize * height as usize * 4)?;
        let mut source = picture
            .as_chunks::<4>()
            .0
            .iter()
            .map(|d| u32::from_le_bytes(*d));
        let rows = crate::gfx::rows::Rows::new(target, pitch, height);
        for y in 0..height {
            // The row's pixels, `bytes` each, written at once.
            let row: Vec<u8> = match bytes {
                2 => (0..width)
                    .flat_map(|_| {
                        (components_to_pixel(&format, source.next().unwrap_or(0)) as u16)
                            .to_le_bytes()
                    })
                    .collect(),
                4 => (0..width)
                    .flat_map(|_| {
                        components_to_pixel(&format, source.next().unwrap_or(0)).to_le_bytes()
                    })
                    .collect(),
                _ => return Err(fatal(0x80A1_8775)),
            };
            rows.write(y, 0, &row, "a texture's row")?;
        }
        surfaces
            .unlock(surface)
            .map_err(|_| direct_draw_failed(0x8021_8773, gfx))?;
    }
    Ok(())
}
