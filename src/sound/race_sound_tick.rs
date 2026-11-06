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

use crate::errors::Refused;
use crate::physics::tables_store::PHYSICS;
use crate::race::owner_frame::Frame;

// g_sceneClock, dword_5CC3B0 the last tick, dword_5CC3B8 the ticks counted.

// ---- Rust version ----

/// No caller reads what the original leaves in eax and ebx (every call site's live
/// registers, 2026-10-23): the Rust version leaves nothing.
pub fn race_sound_tick(frame: &Frame) -> Result<(), Refused> {
    // Converted: `rc2k_core::sound::turn::tick_due`.
    if !rc2k_core::sound::turn::tick_due(
        PHYSICS.scene_clock.get(),
        frame.clock_ticks.sound_tick_last.get(),
    ) {
        return Ok(());
    }
    frame
        .clock_ticks
        .sound_tick_last
        .set(PHYSICS.scene_clock.get());
    crate::sound::race_turn_if::race_turn_if()?;
    frame
        .clock_ticks
        .sound_ticks
        .set(frame.clock_ticks.sound_ticks.get().wrapping_add(1));
    Ok(())
}
