//! A picture into every level of a system texture.
//!
//! - The picture made ready (`gfx/texture_upload.rs`: the least colour, the alpha from the kept
//!   picture or the colours keyed, the grading).
//! - Picture and texture the same number of halvings from 256 across and down, else a fatal
//!   error (0x80A18741 .. 0x80A18747).
//! - For each of the record's levels (one at least): the picture halved (that many times for the
//!   first, once for each after), the level's surface locked (DirectDraw, into the description at
//!   `size`), every pixel through `components_to_pixel` with the texture's format into it as
//!   words or dwords by the kind's bytes a pixel (another number: fatal error 0x80A18775), rows
//!   `pitch` apart from `pixels`, and unlocked.
//! - A lock or unlock that fails is a fatal error (0x80218772, 0x80218773) when `reports.on` is
//!   1; otherwise it is refused.
//!
//! One caller, `gfx::load_picture_to_texture`. The two DirectDraw calls go through `Surfaces`,
//! which the standalone program can answer with textures of its own. Refuses a texture of 0x140
//! or more, a record without its format, more than nine levels, a picture that halves to
//! nothing, and what `ready_picture` refuses; a refusal after the picture was changed leaves it
//! changed.

use crate::errors::{BadInput, Refused};
use crate::gfx::owner::Gfx;
use crate::gfx::texture_pixels::{halvable, halve_texture};
use crate::gfx::texture_upload::{Record, fatal, halvings_apart, ready_picture};
use rc2k_core::gfx::uploads::{self};

/// A level's surface locked and unlocked; an error is what DirectDraw returned.
pub trait Surfaces {
    /// The pixels and the pitch.
    fn lock(&mut self, surface: u32) -> Result<(u32, u32), i32>;
    fn unlock(&mut self, surface: u32) -> Result<(), i32>;
}

/// DirectDraw's own calls, the description being the game's.
pub struct GameSurfaces<'a>(pub &'a Gfx);

impl Surfaces for GameSurfaces<'_> {
    fn lock(&mut self, surface: u32) -> Result<(u32, u32), i32> {
        let card = &self.0.card;
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
    let _timed = crate::race::load_times::scope(crate::race::load_times::Category::TextureConvert);
    let record = Record::of(texture, gfx)?;
    let levels = uploads::levels(record.get().levels).map_err(BadInput::from)?;
    {
        let _preparing =
            crate::race::load_times::scope(crate::race::load_times::Category::TexturePrepare);
        ready_picture(&record, pixels, width, height, gfx)?;
    }
    let apart = halvings_apart(
        gfx,
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
            let _halving =
                crate::race::load_times::scope(crate::race::load_times::Category::TextureHalve);
            (width, height) = halve_texture(&gfx.pictures.buffers, pixels, width, height, times)?;
        }
        let surface = record.get().surfaces[level as usize];
        let (target, pitch) = {
            let _locking =
                crate::race::load_times::scope(crate::race::load_times::Category::SurfaceLock);
            surfaces
                .lock(surface)
                .map_err(|_| direct_draw_failed(0x8021_8772, gfx))?
        };
        // The level's pixels read once; the surface is the platform's.
        let picture = crate::gfx::picture::read_bytes(
            &gfx.pictures.buffers,
            pixels,
            0,
            width as usize * height as usize * 4,
        )?;
        if !matches!(bytes, 2 | 4) {
            return Err(fatal(0x80A1_8775));
        }
        let rows = crate::gfx::rows::Rows::new(target, pitch, height);
        let row_bytes = width as usize * bytes as usize;
        let mut row = vec![0u8; row_bytes];
        for (y, components) in (0..height).zip(picture.chunks_exact(width as usize * 4)) {
            // The row's pixels, `bytes` each, written at once; the bit-by-bit formats leave their
            // scratch bytes as the row's last pixel did.
            let kept = rc2k_core::gfx::pixels::pack_row(
                components,
                &mut row,
                width as usize,
                &format,
                bytes == 2,
            )
            .map_err(BadInput::from)?;
            if let Some(scratch) = kept {
                gfx.pixel.formats.scratch.write(&scratch);
            }
            #[cfg(feature = "remote")]
            crate::race::course_hash::note_texture_row(texture, level, y, &row);
            rows.write(y, 0, &row, "a texture's row")?;
        }
        {
            let _unlocking =
                crate::race::load_times::scope(crate::race::load_times::Category::SurfaceUnlock);
            surfaces
                .unlock(surface)
                .map_err(|_| direct_draw_failed(0x8021_8773, gfx))?;
        }
    }
    Ok(())
}
