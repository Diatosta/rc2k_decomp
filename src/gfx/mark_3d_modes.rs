//! `gfx::mark_3d_modes` (0x40442C): the screen modes Direct3D can draw in, marked in the
//! modes' table.
//!
//! Nothing unless `machine.has_fpu` (set by `system::test_processor`) is 1.
//! First Direct3D's devices are looked at ("D3D: detect card"): the depth formats go to
//! `mode.depth_bits` and `mode.depth_extra`, and the carry says none fit (the platform's
//! version is below). Then over the modes' table (`modes.count` entries of 8 bytes: flags,
//! a word of extra flags): the next entry with all of the card kind's bits
//! (`system::find_entry_with_bits`), from there the next with any of the depth bits
//! (`system::find_entry_with_any_bit`); that one gets 0x100000 and, with `depth_extra` not
//! 0, 0x200000 and `depth_extra` in its second dword; one entry on (`system::next_entry`,
//! its carry not looked at), again, until a search finds nothing. Refuses (before
//! anything) more than 0x20 entries, the table's room: past it the original would mark
//! what lies after.
//! In the standalone the device is its rasteriser: it draws in both pixel kinds with a
//! depth buffer of 16 or 32 bits and has every capability the game asks about (the depth
//! formats and the `cards.mode_switches` that the graphics options read), so one always
//! fits.
//! Callers: `gfx::list_primary_modes`, `gfx::list_secondary_modes`.

use crate::d3d::owner::D3d;
use crate::errors::{BadInput, Refused};
use crate::gfx::owner::Display;
use crate::system::owner::System;

pub use rc2k_core::gfx::screen_modes::MODES_ROOM;

/// Marks the modes listed for card `kind` (see the header).
pub fn mark_3d_modes(
    display: &Display,
    system: &System,
    d3d: &D3d,
    kind: u32,
) -> Result<(), Refused> {
    if system.session.machine.has_fpu.get() != 1 {
        return Ok(());
    }
    let count: u32 = display.modes.count.get();
    if count > MODES_ROOM {
        return Err(BadInput("more screen modes than the table has room for").into());
    }
    // The device: 16 and 32 bits (`g_pixelKinds`' 0x200 and 0x800) for both.
    display.mode.depth_bits.set(0xA00);
    display.mode.depth_extra.set(0xA00);
    // Every capability switch is on.
    let cards = d3d.cards.get();
    for cell in cards.mode_switches.iter().chain(&cards.mode_switches_end) {
        cell.set(1);
    }
    cards.fog_state.set(1);
    cards.mips.set(1);
    // Converted: `rc2k_core::gfx::screen_modes::mark_3d`.
    let before: Vec<rc2k_core::system::tables::Entry> = (0..count as usize)
        .map(|n| {
            [
                display.modes.table[2 * n].get(),
                display.modes.table[2 * n + 1].get(),
            ]
        })
        .collect();
    let mut marked = before.clone();
    rc2k_core::gfx::screen_modes::mark_3d(
        &mut marked,
        kind,
        display.mode.depth_bits.get(),
        display.mode.depth_extra.get(),
    );
    for (n, e) in marked.into_iter().enumerate() {
        if e != before[n] {
            display.modes.table[2 * n].set(e[0]);
            display.modes.table[2 * n + 1].set(e[1]);
        }
    }
    Ok(())
}
