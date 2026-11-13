//! Makes four 8-bit components a pixel of the surface's format and puts it at x, y on the surface,
//! returning the pixel.
//! The format comes through the surface, `gfx::components_to_pixel`; the pixel is 16 bits wide when
//! the surface's bits per pixel are 16, else 32. Refuses no format (with components not 0) and no
//! surface.

use crate::errors::{BadInput, Refused};
use crate::gfx::components_to_pixel::components_to_pixel;

/// `components` made a pixel and put at (`x`, `y`); the pixel.
pub fn put_surface_pixel(
    components: u32,
    x: u32,
    y: u32,
    gfx: &crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
) -> Result<u32, Refused> {
    let drawing = &gfx.drawing;
    // The surface's format pointer.
    let format: u32 = drawing.draw_surface.format.get();
    if components != 0 && format == 0 {
        return Err(BadInput("no surface format").into());
    }
    if crate::gfx::surface::handle(gfx)? == 0 {
        return Err(BadInput("no surface").into());
    }
    let pixel = if components == 0 {
        0
    } else {
        components_to_pixel(
            &crate::gfx::pixel_format::layout(system, gfx, format)?,
            components,
            gfx,
        )
    };
    let (at, wide) =
        rc2k_core::gfx::picture_size::pixel_place(drawing.draw_surface.bits_per_pixel.get(), x);
    let place = crate::gfx::rows::Rows::surface(gfx)?.at(y, at)?;
    if wide == 2 {
        place.write(pixel as u16, "a surface's pixel")?;
    } else {
        place.write(pixel, "a surface's pixel")?;
    }
    Ok(pixel)
}
