//! `sound::race_sound_tick` (0x421BA9): the race frame's sound tick: every 20 ms of the
//! scene clock, `sound::race_turn_if`.
//!
//! `g_sceneClock` less `dword_5CC3B0` under 0x14 (unsigned): nothing. Else
//! `dword_5CC3B0` = the clock, `sound::race_turn_if`, `dword_5CC3B8` counts up.
//! 28 callers, all in the race's frame (`drive::race_frame`, `race::draw_race_scene`,
//! `race::show_frame`,
//! `race::draw_view_scene`, `race::draw_view_overlays`, 0x442D49): for the Rust ones, the Rust
//! version (`race_sound_tick`); `race_turn`'s quit jump then
//! skips their frames (held for the owner: notes/open_items_full.md).

use crate::errors::Refused;
use crate::race::owner::race;
use crate::race::owner_frame::Frame;

// g_sceneClock, dword_5CC3B0 the last tick, dword_5CC3B8 the ticks counted.

/// Plays the race sounds once the sound clock says a tick is due.
pub fn race_sound_tick(frame: &Frame) -> Result<(), Refused> {
    let input = crate::input::owner::input();
    let sound = crate::sound::owner::sound();
    let race = race();
    // Converted: `rc2k_core::sound::turn::tick_due`.
    if !rc2k_core::sound::turn::tick_due(
        race.shared.physics_tables.scene_clock.get(),
        frame.clock_ticks.sound_tick_last.get(),
    ) {
        return Ok(());
    }
    frame
        .clock_ticks
        .sound_tick_last
        .set(race.shared.physics_tables.scene_clock.get());
    crate::sound::race_turn_if::race_turn_if(sound, input)?;
    frame
        .clock_ticks
        .sound_ticks
        .set(frame.clock_ticks.sound_ticks.get().wrapping_add(1));
    Ok(())
}
