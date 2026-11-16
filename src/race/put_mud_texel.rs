//! `race::put_mud_texel`: sets a texel of the locked mud texture (`surface`) by its place on a
//! 256 x 256 square, the place halved as in `race::mud_texel`; skipped when an odd place is
//! halved, or outside the surface; else `gfx::put_surface_pixel`.
//!
//! Takes the components and x, y (0 .. 0xFF); gives the pixel put, or the one passed in.
//!
//! See `race/mud.rs`. Refuses what `put_surface_pixel` refuses (no format, no surface), before
//! anything.

use crate::errors::Refused;
use crate::gfx::put_surface_pixel::put_surface_pixel;
use crate::race::mud_texel::texel_place;

pub fn put_mud_texel(
    components: u32,
    x: u32,
    y: u32,
    scene: &crate::scene::owner::Scene,
    gfx: &crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
) -> Result<u32, Refused> {
    let drawing = &gfx.drawing;
    let Some((x, y)) = texel_place(x, y, scene) else {
        return Ok(components);
    };
    // Logic in `rc2k_core::race::mud::inside` (the place by `texel_place`).
    if !rc2k_core::race::mud::inside(
        x,
        y,
        drawing.draw_surface.width.get(),
        drawing.draw_surface.height.get(),
    ) {
        return Ok(components);
    }
    put_surface_pixel(components, x, y, gfx, system)
}
