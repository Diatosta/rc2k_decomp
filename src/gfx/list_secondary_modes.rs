//! `gfx::list_secondary_modes` (0x404401): with a 3D secondary card (`cards.secondary` 1,
//! `gfx::find_3d_cards`' "secondary card"), its screen modes listed into the modes' table
//! and marked for Direct3D; DirectDraw closed in any case.
//!
//! With the card: DirectDraw is made for the card's GUID, its modes are enumerated with
//! the secondary card's bit (0x2000) and marked with `gfx::mark_3d_modes` for the same
//! kind. Then DirectDraw is closed. DirectDraw as in `gfx::list_primary_modes`. One
//! caller, `start`, which calls it directly.

use crate::errors::Refused;
use crate::gfx::list_primary_modes::standalone_modes;
use crate::gfx::mark_3d_modes::mark_3d_modes;
use crate::gfx::owner::Display;

pub const SECONDARY: u32 = 0x2000;
pub fn list_secondary_modes(display: &Display) -> Result<(), Refused> {
    if display.cards.secondary.get() == 1 {
        standalone_modes(display, SECONDARY)?;
        mark_3d_modes(display, SECONDARY)?;
    }
    Ok(())
}
