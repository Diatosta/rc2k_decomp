//! `gfx::list_primary_modes` (0x4043DA): the primary card's screen modes listed into the
//! modes' table and marked for Direct3D. (`gfx::list_secondary_modes` is the same for the
//! secondary card; the two share what is here.)
//!
//! The primary: `sub_4048EC` with eax 0 (DirectDraw made for the card, "VID: Open ddraw";
//! failing, fatal errors 0x80208511 / 0x80208512, raised there), `sub_404A31` with eax
//! 0x1000 (DirectDraw's `EnumDisplayModes` with `sub_404A56`, which puts the modes in the
//! table), `gfx::mark_3d_modes` with eax 0x1000 if `dword_5AB4AC` (a 3D primary card) is
//! 1, then `sub_404935` (DirectDraw closed). DirectDraw is the platform's: in the standalone
//! the modes are every size the game knows in both pixel kinds (`standalone_modes`), and
//! the opening and closing nothing. One caller, `start` (0x401098), which calls it directly
//! (2026-10-23).

use crate::errors::{BadInput, Refused};
use crate::gfx::mark_3d_modes::mark_3d_modes;
use crate::gfx::owner::Display;

pub const PRIMARY: u32 = 0x1000;

/// The Rust version (see the header).
pub fn list_primary_modes(display: &Display) -> Result<(), Refused> {
    standalone_modes(display, PRIMARY)?;
    if display.cards.primary.get() == 1 {
        mark_3d_modes(display, PRIMARY)?;
    }
    Ok(())
}

const TABLE_ROOM: u32 = 0x20;

/// The standalone program's DirectDraw `EnumDisplayModes` for card `card` (0x1000 the
/// primary): every screen size the game knows in both pixel kinds (its window takes
/// any), entered as `gfx::mode_callback` (0x404A56) enters a mode: the card's, the kind's
/// and the size's bits, once each, while the table has room.
pub fn standalone_modes(display: &Display, card: u32) -> Result<(), BadInput> {
    let table = &display.modes.table;
    for size in 0..8usize {
        let size_bit: u32 = display.modes.sizes[2 * size].get();
        for kind in 0..2usize {
            let kind_bit: u32 = display.modes.pixel_kinds[2 * kind].get();
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
    }
    Ok(())
}
