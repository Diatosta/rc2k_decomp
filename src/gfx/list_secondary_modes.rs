//! With a 3D secondary card (`cards.secondary` 1, `gfx::find_3d_cards`' "secondary card"), lists
//! its screen modes into the modes' table and marks them for Direct3D; DirectDraw is closed in any
//! case.
//! With the card: DirectDraw is made for the card's GUID, its modes are enumerated with the
//! secondary card's bit (0x2000) and marked with `gfx::mark_3d_modes` for the same kind. Then
//! DirectDraw is closed, as in `gfx::list_primary_modes`.

use crate::errors::Refused;
use crate::gfx::list_primary_modes::standalone_modes;
use crate::gfx::mark_3d_modes::mark_3d_modes;
use crate::gfx::owner::Display;

pub const SECONDARY: u32 = 0x2000;
pub fn list_secondary_modes(
    display: &Display,
    system: &crate::system::owner::System,
    d3d: &crate::d3d::owner::D3d,
) -> Result<(), Refused> {
    if display.cards.secondary.get() == 1 {
        standalone_modes(display, SECONDARY)?;
        mark_3d_modes(display, system, d3d, SECONDARY)?;
    }
    Ok(())
}
