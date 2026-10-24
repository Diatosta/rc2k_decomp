//! `sound::race_sound_tick` (0x421BA9): the race frame's sound tick: every 20 ms of the
//! scene clock, `sound::race_turn_if`.
//!
//! In:  nothing
//! Out: eax the scene clock less the last tick (not due) or the clock (due); ebx as
//!      `race_turn_if` leaves it; everything else preserved
//! `g_sceneClock` less `dword_5CC3B0` under 0x14 (unsigned): nothing. Else
//! `dword_5CC3B0` = the clock, `sound::race_turn_if`, `dword_5CC3B8` counts up.
//! 28 callers, all in the race's frame (`sub_4541BF`, `sub_45429B`, `sub_4547AE`,
//! `sub_4557AB`, `sub_4559A6`, `sub_442D49`): for the Rust ones, the Rust version
//! (`race_sound_tick`); `race_turn`'s quit jump then
//! skips their frames (held for the owner: notes/open_items_full.md).
//! Ungated: the quit jump never comes back.
//! Shadow mode runs the original alone (sounds are sent).
//! Self-test: a copy whose call goes to a stand-in, against the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{game_write, rva};

/// g_sceneClock, dword_5CC3B0 the last tick, dword_5CC3B8 the ticks counted.
const CLOCK: usize = 0x613830;
const LAST: usize = 0x5CC3B0;
const TICKS: usize = 0x5CC3B8;
fn g(x: impl crate::store::Ida) -> u32 {
    unsafe { crate::store::ida_read(x) }
}

// ---- Rust version ----

/// No caller reads what the original leaves in eax and ebx (every call site's live
/// registers, 2026-10-23): the Rust version leaves nothing.
pub fn race_sound_tick() -> Result<(), Refused> {
    // Converted: `rc2k_core::sound::turn::tick_due`.
    if !rc2k_core::sound::turn::tick_due(g(CLOCK), g(LAST)) {
        return Ok(());
    }
    unsafe { game_write(rva(LAST), g(CLOCK)) };
    crate::sound::race_turn_if::race_turn_if()?;
    unsafe { game_write(rva(TICKS), g(TICKS).wrapping_add(1)) };
    Ok(())
}
