//! `gfx::blit_components` (0x4065EE): a piece of a picture of components onto the
//! surface, in the surface's pixel format.
//!
//! In:  eax = the picture's bytes from one row to the next, ebx = the piece (x, y,
//!      width, height in the picture), ecx, edx = where on the surface, esi = the
//!      picture's pixels
//! Out: every register preserved
//!
//! The surface's place = its pixels (`g_surface.pixels`) + y times its pitch + x times its
//! bytes a pixel; the picture's = esi + the piece's y times eax + its x times 4. Each row: the
//! piece's width of pixels through `gfx::components_to_pixel` with the surface's format
//! (`g_surface.format`), stored as words on a surface of two bytes a pixel, else as dwords;
//! then both move on a row. No clipping: a row past the surface's last, or bytes past its
//! last row, are refused (`gfx::rows`). One caller, `scene::upload_car_icon`, with the texture
//! made the surface. Refuses a piece of no width or height (the original counts through all 32
//! bits), and no surface, format, piece or picture.

use crate::errors::{BadInput, Refused};
use crate::gfx::formats::Formats;

// ---- Rust version ----

/// The surface as the function reads it.
#[derive(Clone, Copy, Debug)]
pub struct Target {
    pub rows: crate::gfx::rows::Rows,
    pub bytes_per_pixel: u32,
    pub format: u32,
}

/// The surface (a texture: refused while it is the picture buffer).
pub fn game_target() -> Result<Target, BadInput> {
    Ok(Target {
        rows: crate::gfx::rows::Rows::surface()?,
        bytes_per_pixel: crate::gfx::owner::gfx()
            .drawing
            .draw_surface
            .bytes_per_pixel
            .get(),
        // `g_surface.format`: the address of the surface's `PixelFormat`.
        format: crate::gfx::owner::gfx().drawing.draw_surface.format.get(),
    })
}

/// What the original gets in its registers: ebx the piece's rectangle (x, y, width,
/// height), ecx, edx where it goes on the surface, esi the picture's pixels (a picture
/// buffer's number since phase 12, `gfx::picture`), eax the picture's row bytes.
#[derive(Clone, Copy, Debug)]
pub struct Piece {
    pub rectangle: crate::pointers::GamePtr,
    pub x: u32,
    pub y: u32,
    pub pixels: u32,
    pub row_bytes: u32,
}

/// Whether there is something to blit: a piece with a rectangle of some width and height,
/// a picture, a surface and its format.
fn something(p: &Piece, to: &Target) -> bool {
    if p.rectangle.is_null() || p.pixels == 0 || to.rows.is_null() || to.format == 0 {
        return false;
    }
    let [_, _, width, height]: [u32; 4] = p
        .rectangle
        .read("a blit's rectangle")
        .unwrap_or_else(|e| crate::fatal::raise::end("a blit's rectangle", e));
    width != 0 && height != 0
}

pub fn blit_components(formats: &Formats, p: &Piece, to: &Target) -> Result<(), Refused> {
    if !something(p, to) {
        return Err(BadInput("a piece of nothing, or no surface, format, piece or picture").into());
    }
    let [x, y, width, height] = p.rectangle.read::<[u32; 4]>("a piece's rectangle")?;
    let format = crate::gfx::pixel_format::layout(to.format)?;
    // The picture's bytes from its buffer (`gfx::picture`; `pixels` names it).
    let mut source = (y.wrapping_mul(p.row_bytes) as usize).wrapping_add(x as usize * 4);
    // Converted: `rc2k_core::gfx::pixels::pack_row`, a row at a time (every byte of the
    // surface's row written).
    let words = to.bytes_per_pixel == 2;
    let size = if words { 2 } else { 4 };
    let mut out = vec![0u8; width as usize * size];
    for n in 0..height {
        let row = crate::gfx::picture::read_bytes(p.pixels, source, width as usize * 4)?;
        let scratch =
            rc2k_core::gfx::pixels::pack_row(&row, &mut out, width as usize, &format, words)?;
        // Row y + n at x's bytes, as the original's y x pitch + x x bytes.
        to.rows.write(
            p.y.wrapping_add(n),
            p.x.wrapping_mul(to.bytes_per_pixel),
            &out,
            "a surface's row",
        )?;
        if let Some(scratch) = scratch {
            formats.scratch.write(&scratch);
        }
        source = source.wrapping_add(p.row_bytes as usize);
    }
    Ok(())
}
