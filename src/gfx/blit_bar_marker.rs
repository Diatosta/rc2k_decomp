//! Draws one sprite at the point of a bar that a percentage reaches, and moves the bar's x there.
//! Above 0 percent, a width * percent of 0 is a fatal error, and the palette given to the blitter
//! is the marker's x. These sprites do not use a palette.

use crate::errors::Refused;
use crate::gfx::blit_bar::Blit;
use crate::gfx::owner::Gfx;
use crate::menu::owner::Menus;

/// The marker of the bar the bar globals describe.
pub fn blit_bar_marker(gfx: &Gfx, menus: &Menus, blit: Blit) -> Result<(), Refused> {
    use crate::gfx::blit_bar::{Bar, GameBar};
    rc2k_core::gfx::bars::marker(&mut GameBar { blit, gfx, menus }, &Bar::read(gfx).core())
}
