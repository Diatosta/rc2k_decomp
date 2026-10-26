//! `race::put_mud_texel` (0x44A5A9): a texel of the locked mud texture (`g_surface`) set
//! by its place on a 256 x 256 square, the place halved as `race::mud_texel`'s; skipped
//! when an odd place is halved, or outside the surface; else `gfx::put_surface_pixel`.
//!
//! In:  eax = the components, ecx = x, edx = y (0 .. 0xFF)
//! Out: eax = the pixel put, or as it came; the rest kept; no flags read
//!
//! See `race/mud.rs`. Callers: `race::muddy_texel`, `race::splash_mud`. Refuses what
//! `put_surface_pixel` refuses (no format, no surface), before anything.

use crate::errors::Refused;
use crate::gfx::put_surface_pixel::put_surface_pixel;
use crate::gfx::screen_state::DRAW_SURFACE;
use crate::race::mud_texel::texel_place;

/// The Rust version (see the header): eax as it leaves it.
pub unsafe fn put_mud_texel(components: u32, x: u32, y: u32) -> Result<u32, Refused> {
    let Some((x, y)) = texel_place(x, y) else {
        return Ok(components);
    };
    unsafe {
        // Converted: `rc2k_core::race::mud::inside` (the place by `texel_place`).
        if !rc2k_core::race::mud::inside(x, y, DRAW_SURFACE.width.get(), DRAW_SURFACE.height.get())
        {
            return Ok(components);
        }
        put_surface_pixel(components, x, y)
    }
}
