//! `gfx::mark_3d_modes` (0x40442C): the screen modes Direct3D can draw in, marked in the
//! modes' table.
//!
//! In:  eax = the kind of card the modes were listed for (0x1000 the primary, 0x2000 the
//!      secondary: `gfx::list_primary_modes`, `gfx::list_secondary_modes`)
//! Out: every register preserved (`pusha` .. `popa`)
//! Nothing unless `g_hasFpu` (0x5183EC, set by `system::test_processor`) is 1.
//! `sub_419853` (Direct3D's devices looked at, "D3D: detect card", the depth formats into
//! `dword_4F492C` and `dword_4F4930`; the carry: none fit) the platform's (below). Then over the
//! table `dword_4F4F3C` (`dword_4F4F38` entries of 8 bytes: flags, a word of extra flags):
//! the next entry with all of eax's bits (`system::find_entry_with_bits`), from there the
//! next with any of `dword_4F492C`'s (`system::find_entry_with_any_bit`); that one gets
//! 0x100000 and, with `dword_4F4930` not 0, 0x200000 and `dword_4F4930` in its second
//! dword; one entry on (`system::next_entry`, its carry not looked at), again, until a
//! search finds nothing. Refuses (before anything) more than 0x20 entries, the table's
//! room (`sub_404A56` stops there): past it the original would mark what lies after.
//! In the standalone the device is its rasteriser: it draws in both pixel kinds with a
//! depth buffer of 16 or 32 bits and has every capability the game asks about
//! (`sub_419853`'s results: `g_depthFormatBits`, `g_depthExtraFlags` and the `caps3d..`
//! switches 0x5AB98C .. 0x5AB9AC that the graphics options read), so one always fits.
//! Callers: `gfx::list_primary_modes`, `gfx::list_secondary_modes`.

use crate::errors::{BadInput, Refused};
use crate::framework::{game_read, game_write, rva};

pub use rc2k_core::gfx::screen_modes::MODES_ROOM;

const HAS_FPU_RVA: usize = rva(0x5183EC); // g_hasFpu
const TABLE_RVA: usize = rva(0x4F4F3C); // dword_4F4F3C: the table
const COUNT_RVA: usize = rva(0x4F4F38); // dword_4F4F38: its entries
const DEPTH_RVA: usize = rva(0x4F492C); // dword_4F492C: the depth formats' bits
const EXTRA_RVA: usize = rva(0x4F4930); // dword_4F4930: the extra flags

/// The Rust version on the modes listed for card `kind` (see the header).
pub fn mark_3d_modes(kind: u32) -> Result<(), Refused> {
    unsafe {
        if game_read::<u32>(HAS_FPU_RVA) != 1 {
            return Ok(());
        }
        let count: u32 = game_read(COUNT_RVA);
        if count > MODES_ROOM {
            return Err(BadInput("more screen modes than the table has room for").into());
        }
        // The device: 16 and 32 bits (`g_pixelKinds`' 0x200 and 0x800) for both.
        game_write(DEPTH_RVA, 0xA00u32);
        game_write(EXTRA_RVA, 0xA00u32);
        for at in (0x5AB98C..=0x5AB9AC).step_by(4) {
            game_write(rva(at), 1u32);
        }
        // Converted: `rc2k_core::gfx::screen_modes::mark_3d`.
        let table = crate::framework::game_addr(TABLE_RVA) as u32;
        let before: Vec<_> = crate::system::find_entry::entries(table, count).collect();
        let mut marked = before.clone();
        rc2k_core::gfx::screen_modes::mark_3d(
            &mut marked,
            kind,
            game_read(DEPTH_RVA),
            game_read(EXTRA_RVA),
        );
        for (n, e) in marked.into_iter().enumerate() {
            if e != before[n] {
                ((table + 8 * n as u32) as *mut rc2k_core::system::tables::Entry)
                    .write_unaligned(e);
            }
        }
    }
    Ok(())
}
