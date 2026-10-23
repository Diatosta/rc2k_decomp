//! `gfx::list_secondary_modes` (0x404401): with a 3D secondary card (`dword_5AB4B0` 1,
//! `gfx::find_3d_cards`' "secondary card"), its screen modes listed into the modes' table
//! and marked for Direct3D; DirectDraw closed in any case.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//! With the card: `sub_4048EC` with eax its GUID (`unk_5AB4B4`), `sub_404A31` with eax
//! 0x2000, `gfx::mark_3d_modes` with eax 0x2000. Then `sub_404935` (DirectDraw closed).
//! DirectDraw as in `gfx::list_primary_modes`. One caller, `start` (0x40109D), which calls
//! it directly (2026-10-23).

use crate::errors::Refused;
use crate::framework::{game_read, rva};
use crate::gfx::list_primary_modes::standalone_modes;
use crate::gfx::mark_3d_modes::mark_3d_modes;

pub const SECONDARY: u32 = 0x2000;
const SECONDARY_3D_RVA: usize = rva(0x5AB4B0); // dword_5AB4B0: a 3D secondary card

/// The Rust version (see the header).
pub fn list_secondary_modes() -> Result<(), Refused> {
    if unsafe { game_read::<u32>(SECONDARY_3D_RVA) } == 1 {
        unsafe { standalone_modes(SECONDARY) };
        mark_3d_modes(SECONDARY)?;
    }
    Ok(())
}
