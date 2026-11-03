//! `gfx::blit_bar_marker` (0x46B5FC): draws one sprite at the point of a bar that a percentage
//! reaches, and moves the bar's x there.
//!
//! In:  nothing in registers except ebx, which is passed on as the palette at 0 percent or
//!      less; the bar is described by the bar globals (see `blit_bar::Bar`)
//! Out: every register preserved; above 0 percent the bar's x (0x755224) is the marker's x
//! Above 0 percent, a width * percent of 0 is fatal error 0x80701010 in the original, and the
//! palette given to the blitter is the marker's x. These sprites do not use a palette.

use crate::errors::Refused;
use crate::gfx::blit_bar::Blit;

// ---- Rust version ----

/// The marker of the bar the bar globals describe.
pub fn blit_bar_marker(blit: Blit) -> Result<(), Refused> {
    // Converted: `rc2k_core::gfx::bars::marker`.
    use crate::gfx::blit_bar::{Bar, GameBar};
    rc2k_core::gfx::bars::marker(&mut GameBar(blit), &Bar::read().core())
}
