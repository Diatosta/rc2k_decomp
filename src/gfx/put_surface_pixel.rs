//! `gfx::put_surface_pixel` (0x4065BB): four 8-bit components made a pixel of the surface's
//! format and put at x, y on `g_surface`.
//!
//! In:  eax the components, ecx x, edx y
//! Out: eax the pixel; everything else kept
//!
//! The format through `g_surface`'s pointer (0x4F49B8), `gfx::components_to_pixel`; the pixel
//! 16 bits wide when the surface's bits per pixel are 16, else 32. Five callers (among them the
//! marks' snow textures). Refuses no format (with components not 0) and no surface.

use crate::errors::{BadInput, Refused};
use crate::gfx::components_to_pixel::components_to_pixel;
use crate::gfx::screen_state::DRAW_SURFACE;

/// Where the pixel goes and how wide it is.
pub fn place(x: u32, y: u32) -> Result<(u32, u32), BadInput> {
    // Converted: `rc2k_core::gfx::picture_size::pixel_place`.
    Ok(rc2k_core::gfx::picture_size::pixel_place(
        crate::gfx::surface::address()?,
        DRAW_SURFACE.pitch.get(),
        DRAW_SURFACE.bits_per_pixel.get(),
        x,
        y,
    ))
}

/// The Rust version (see the header): `components` made a pixel and put at (`x`, `y`);
/// the pixel (the original's eax).
pub fn put_surface_pixel(components: u32, x: u32, y: u32) -> Result<u32, Refused> {
    use crate::pointers::GamePtr;
    // `g_surface`'s format pointer.
    let format: u32 = DRAW_SURFACE.format.get();
    if components != 0 && format == 0 {
        return Err(BadInput("no surface format").into());
    }
    if crate::gfx::surface::address()? == 0 {
        return Err(BadInput("no surface").into());
    }
    let pixel = if components == 0 {
        0
    } else {
        components_to_pixel(&crate::gfx::pixel_format::layout(format)?, components)
    };
    let (at, wide) = place(x, y)?;
    if wide == 2 {
        GamePtr::new(at).write(pixel as u16, "a surface's pixel")?;
    } else {
        GamePtr::new(at).write(pixel, "a surface's pixel")?;
    }
    Ok(pixel)
}
