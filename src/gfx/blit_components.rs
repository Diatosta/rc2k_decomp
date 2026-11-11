//! `gfx::blit_components` (0x4065EE): a piece of a picture of components onto the
//! surface, in the surface's pixel format.
//!
//! The surface's place = its pixels (`g_surface.pixels`) + y times its pitch + x times its
//! bytes a pixel; the picture's = its pixels + the piece's y times its row bytes + its x times 4. Each row: the
//! piece's width of pixels through `gfx::components_to_pixel` with the surface's format
//! (`g_surface.format`), stored as words on a surface of two bytes a pixel, else as dwords;
//! then both move on a row. No clipping: a row past the surface's last, or bytes past its
//! last row, are refused (`gfx::rows`). One caller, `scene::upload_car_icon`, with the texture
//! made the surface. Refuses a piece of no width or height (the original counts through all 32
//! bits), and no surface, format, piece or picture.

use crate::errors::{BadInput, Refused};
use crate::gfx::formats::Formats;

/// The surface as the function reads it.
#[derive(Clone, Copy, Debug)]
pub struct Target {
    pub rows: crate::gfx::rows::Rows,
    pub bytes_per_pixel: u32,
    pub format: u32,
}

/// The surface (a texture: refused while it is the picture buffer).
pub fn game_target(gfx: &crate::gfx::owner::Gfx) -> Result<Target, BadInput> {
    let drawing = &gfx.drawing;
    Ok(Target {
        rows: crate::gfx::rows::Rows::surface()?,
        bytes_per_pixel: drawing.draw_surface.bytes_per_pixel.get(),
        // `g_surface.format`: the address of the surface's `PixelFormat`.
        format: drawing.draw_surface.format.get(),
    })
}

/// What is drawn: the piece's rectangle (x, y, width, height), where it goes on the
/// surface, the picture's pixels (a picture buffer's number, `gfx::picture`) and the
/// picture's row bytes.
#[derive(Clone, Copy, Debug)]
pub struct Piece {
    pub rectangle: [u32; 4],
    pub x: u32,
    pub y: u32,
    pub pixels: u32,
    pub row_bytes: u32,
}

/// Whether there is something to blit: a piece with a rectangle of some width and height,
/// a picture, a surface and its format.
fn something(p: &Piece, to: &Target) -> bool {
    if p.pixels == 0 || to.rows.is_null() || to.format == 0 {
        return false;
    }
    let [_, _, width, height] = p.rectangle;
    width != 0 && height != 0
}

pub fn blit_components(formats: &Formats, p: &Piece, to: &Target) -> Result<(), Refused> {
    if !something(p, to) {
        return Err(BadInput("a piece of nothing, or no surface, format, piece or picture").into());
    }
    let [x, y, width, height] = p.rectangle;
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
