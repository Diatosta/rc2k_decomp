//! `sound::race_sound_tick`: the race frame's sound tick: every 20 ms of the scene clock,
//! `sound::race_turn_if`.
//!
//! Less than 0x14 (unsigned) since `ClockTicks.sound_tick_last`: nothing. Else
//! `ClockTicks.sound_tick_last` is set to the clock and `sound::race_turn_if` runs.

use crate::errors::Refused;
use crate::race::owner_frame::Frame;

/// Plays the race sounds once the sound clock says a tick is due.
pub fn race_sound_tick(frame: &Frame, o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    let race = o.race;
    // The logic is `rc2k_core::sound::turn::tick_due`.
    if !rc2k_core::sound::turn::tick_due(
        race.shared.run().scene_clock,
        frame.clock_ticks().sound_tick_last,
    ) {
        return Ok(());
    }
    frame.clock_ticks_mut().sound_tick_last = race.shared.run().scene_clock;
    crate::sound::race_turn_if::race_turn_if(o, race)?;
    Ok(())
}
