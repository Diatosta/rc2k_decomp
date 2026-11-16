//! Lists the primary card's screen modes into the modes' table and marks them for Direct3D.
//! DirectDraw is made for the card ("VID: Open ddraw"; failing is a fatal error), its display modes
//! are enumerated with the primary card's bit (0x1000), which puts the modes in the table,
//! `gfx::mark_3d_modes` runs for the same kind (the one card always draws in 3D), then
//! DirectDraw is closed. DirectDraw is the platform's: the modes are every size
//! the game knows at 32 bits a pixel (`standalone_modes`), and the opening and closing do nothing.

use crate::errors::{BadInput, Refused};
use crate::gfx::mark_3d_modes::mark_3d_modes;
use crate::gfx::owner::Display;

pub const PRIMARY: u32 = 0x1000;
pub fn list_primary_modes(
    display: &Display,
    system: &crate::system::owner::System,
    d3d: &crate::d3d::owner::D3d,
) -> Result<(), Refused> {
    standalone_modes(display, PRIMARY)?;
    mark_3d_modes(display, system, d3d, PRIMARY)?;
    Ok(())
}

const TABLE_ROOM: u32 = 0x20;

/// The standalone program's enumeration of display modes for card `card` (0x1000 the primary):
/// Every screen size the game knows at 32 bits a pixel (its window takes any), entered as
/// `gfx::mode_callback` enters a mode: the card's, the kind's and the size's bits, once each, while
/// the table has room.
pub fn standalone_modes(display: &Display, card: u32) -> Result<(), BadInput> {
    let table = &display.modes.table;
    // The 32-bit pixel kind alone (the second of the two): the screen is never 16 bits.
    let kind_bit: u32 = display.modes.pixel_kinds[2].get();
    for size in 0..8usize {
        let size_bit: u32 = display.modes.sizes[2 * size].get();
        let mode = card | kind_bit | size_bit;
        let count: u32 = display.modes.count.get();
        if count > TABLE_ROOM {
            return Err(BadInput("more screen modes than the table has room for"));
        }
        if (0..count as usize).any(|n| table[2 * n].get() & mode == mode) {
            continue;
        }
        if count >= TABLE_ROOM {
            return Ok(());
        }
        table[2 * count as usize].set(mode);
        display.modes.count.set(count + 1);
    }
    Ok(())
}
