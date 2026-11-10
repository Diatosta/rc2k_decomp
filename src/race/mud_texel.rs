//! `race::mud_texel` (0x44A549): a texel of the locked mud texture (`g_surface`) by its place on a
//! 256 x 256 square: with `dword_5CEEE0` not 0 the place is halved (x, then y; an odd one skips),
//! and halved again unless it is 1 (a reading: how many times the texture is smaller); then
//! `race::mud_pixel` (0 outside the surface).
//!
//! Takes: x, y (0 .. 0xFF)
//!
//! See `race/mud.rs`. Callers: `race::muddy_texel`, `race::splash_mud`.

use crate::errors::BadInput;
use crate::race::mud::mud_pixel;
use crate::scene::owner::scene;

/// The texel's place on the texture, or None when it is skipped (an odd place halved).
pub fn texel_place(x: u32, y: u32) -> Option<(u32, u32)> {
    // Converted: `rc2k_core::race::mud::texel_place`.
    rc2k_core::race::mud::texel_place(x, y, scene().objects.tables.system_levels[256].get())
}

/// Eax as it leaves it.
pub fn mud_texel(eax: u32, x: u32, y: u32) -> Result<u32, BadInput> {
    match texel_place(x, y) {
        Some((x, y)) => mud_pixel(x, y),
        None => Ok(eax),
    }
}
