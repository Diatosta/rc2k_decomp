//! `gfx::list_primary_modes` (0x4043DA): the primary card's screen modes listed into the
//! modes' table and marked for Direct3D. (`gfx::list_secondary_modes` is the same for the
//! secondary card; the two share what is here.)
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//! The primary: `sub_4048EC` with eax 0 (DirectDraw made for the card, "VID: Open ddraw";
//! failing, fatal errors 0x80208511 / 0x80208512, raised there), `sub_404A31` with eax
//! 0x1000 (DirectDraw's `EnumDisplayModes` with `sub_404A56`, which puts the modes in the
//! table), `gfx::mark_3d_modes` with eax 0x1000 if `dword_5AB4AC` (a 3D primary card) is
//! 1, then `sub_404935` (DirectDraw closed). DirectDraw is the platform's: in the standalone
//! the modes are every size the game knows in both pixel kinds (`standalone_modes`), and
//! the opening and closing nothing. One caller, `start` (0x401098), which calls it directly
//! (2026-10-23).

use crate::errors::Refused;
use crate::gfx::block::{game_addr, game_read, game_write};
use crate::gfx::cards::Slot as CardSlot;
use crate::gfx::mark_3d_modes::mark_3d_modes;
use crate::gfx::modes::Slot as ModeSlot;

pub const PRIMARY: u32 = 0x1000;
pub const PRIMARY_3D_RVA: CardSlot = CardSlot::at(0x5AB4AC); // dword_5AB4AC: a 3D primary card

/// The Rust version (see the header).
pub fn list_primary_modes() -> Result<(), Refused> {
    unsafe { standalone_modes(PRIMARY) };
    if unsafe { game_read::<u32>(PRIMARY_3D_RVA) } == 1 {
        mark_3d_modes(PRIMARY)?;
    }
    Ok(())
}

const MODE_TABLE_RVA: ModeSlot = ModeSlot::at(0x4F4F3C); // g_screenModeTable: 8 bytes an entry
const MODE_COUNT_RVA: ModeSlot = ModeSlot::at(0x4F4F38); // g_screenModeCount
const SIZES_RVA: ModeSlot = ModeSlot::at(0x4F4E50); // g_screenSizes: 8 of (bit, width, height)
const PIXEL_KINDS_RVA: ModeSlot = ModeSlot::at(0x4F4E40); // g_pixelKinds: (0x200, 16), (0x800, 32)
const TABLE_ROOM: u32 = 0x20;

/// The standalone program's DirectDraw `EnumDisplayModes` for card `card` (0x1000 the
/// primary): every screen size the game knows in both pixel kinds (its window takes
/// any), entered as `gfx::mode_callback` (0x404A56) enters a mode: the card's, the kind's
/// and the size's bits, once each, while the table has room.
pub unsafe fn standalone_modes(card: u32) {
    unsafe {
        let table = game_addr(MODE_TABLE_RVA) as *mut u32;
        for size in 0..8usize {
            let size_bit: u32 = game_read(SIZES_RVA + 8 * size);
            for kind in 0..2usize {
                let kind_bit: u32 = game_read(PIXEL_KINDS_RVA + 8 * kind);
                let mode = card | kind_bit | size_bit;
                let count: u32 = game_read(MODE_COUNT_RVA);
                if (0..count as usize).any(|n| table.add(2 * n).read_unaligned() & mode == mode) {
                    continue;
                }
                if count >= TABLE_ROOM {
                    return;
                }
                table.add(2 * count as usize).write_unaligned(mode);
                game_write(MODE_COUNT_RVA, count + 1);
            }
        }
    }
}
