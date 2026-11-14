//! Draws the decoded picture onto the surface at the surface's size: `gfx::rescale_picture`'s
//! arithmetic with other ends. The rows are those of the decoded picture where it lies (the decoded
//! stride apart, listed with `gfx::fill_row_table`), and every pixel made goes to the surface in
//! the surface's format: its three parts are put in `components` (whose fourth byte is left as it
//! was), `gfx::components_to_pixel` with the surface's format makes the pixel, and it is stored as
//! a word on a 16-bit surface, else as a dword; the surface's gap is added after each row. Refuses
//! a decoded picture or a surface without width or height, more rows than the row table holds, and
//! missing pixels or format.

use crate::errors::{BadInput, Refused};
use crate::gfx::components_to_pixel::components_to_pixel;
use crate::gfx::fill_row_table::{ROW_TABLE_ROOM, fill_row_table, row_address};
use crate::gfx::rescale_picture::Rescaler;

/// `surface` is the surface drawn on.
pub fn rescale_to_surface(
    rescaler: &Rescaler,
    surface: &crate::gfx::screen_state::DrawSurface,
    gfx: &crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    let pictures = &gfx.pictures;
    let (from_width, from_height): (u32, u32) =
        (pictures.decoder.width.get(), pictures.decoder.height.get());
    let (width, height) = (surface.width.get(), surface.height.get());
    let from: u32 = pictures.decoder.pixels.get();
    if from_width == 0 || from_height == 0 || width == 0 || height == 0 {
        return Err(BadInput("a picture or a surface without width or height").into());
    }
    if from_height > ROW_TABLE_ROOM
        || from == 0
        || surface.pixels.get() == 0
        || surface.format.get() == 0
    {
        return Err(BadInput("a decoded picture or a surface the rescaler cannot take").into());
    }
    // The count was checked above. The rows as offsets in the decoded buffer.
    let _ = fill_row_table(0, pictures.decoder.stride.get(), from_height, gfx);
    let format = crate::gfx::pixel_format::layout(system, gfx, surface.format.get())?;
    let words = surface.bits_per_pixel.get() == 0x10;
    // Where the next pixel goes.
    let mut out = crate::gfx::locked_pixels::Place::start(surface.pixels.get());
    let (width_pixels, gap) = (width, surface.row_gap.get());
    let mut column = 0u32;
    #[cfg(feature = "remote")]
    let mut fold = 0u64;
    let work = crate::gfx::picture::with_buffer(from, |pixels| {
        rc2k_core::gfx::picture::rescale(
            &rc2k_core::gfx::picture::Sizes {
                from_width,
                from_height,
                width,
                height,
            },
            &|row, byte| match pixels.get(row_address(row, gfx).wrapping_add(byte) as usize) {
                Some(&b) => b,
                None => crate::fatal::raise::end(
                    "gfx::rescale_to_surface",
                    BadInput("a pixel read past the decoded picture"),
                ),
            },
            &mut |parts| {
                for (part, &byte) in parts.iter().enumerate() {
                    rescaler.components.set(part, byte);
                }
                let pixel = components_to_pixel(&format, rescaler.components.get_u32(0), gfx);
                #[cfg(feature = "remote")]
                {
                    fold = fold.rotate_left(5).wrapping_mul(0x0100_0000_01B3) ^ u64::from(pixel);
                }
                if words {
                    out.must_write(pixel as u16, "the surface");
                    out.advance(2);
                } else {
                    out.must_write(pixel, "the surface");
                    out.advance(4);
                }
                column += 1;
                if column == width_pixels {
                    column = 0;
                    out.advance(gap);
                }
            },
        )
        .expect("the sizes were checked")
    })?;
    #[cfg(feature = "remote")]
    crate::race::course_hash::note_texture_row(u32::MAX, 0, 0, &fold.to_le_bytes());
    rescaler.step_x.set(work.step_x);
    rescaler.step_x_fraction.set(work.step_x_fraction);
    rescaler.step_y.set(work.step_y);
    rescaler.step_y_fraction.set(work.step_y_fraction);
    rescaler.weight_x.set(work.weight_x);
    rescaler.weight_y.set(work.weight_y);
    rescaler.columns_left.set(0);
    rescaler.rows_left.set(0);
    Ok(())
}
