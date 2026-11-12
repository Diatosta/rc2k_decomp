//! `drive::race_tick` (0x4455A9, three chunks): one tick of the race. Fatal error
//! 0x80600655 (no message) when the first car block's +0x260 is 0 (its third chunk;
//! the second is the shared fatal reporter). Free roaming (`levers_on` 0):
//! `drive::simple_car_step`. Else the tick count `race_ticks` + 1 and
//! `drive::replay_step` every tick, or with `after_credits` set every second tick, and
//! past 1 every fourth.
//!
//! Caller: `drive::race_step`. Raises 0x80600655 as the original; refuses what the steps
//! refuse.

use crate::errors::{FatalError, Refused};
use crate::race::owner::Race;

// Converted: `rc2k_core::drive::race_ticks::{tick_runs, NO_CAR_FATAL}`.
pub fn race_tick(o: crate::owners::Owners<'static>, race: &Race) -> Result<(), Refused> {
    if race.car_blocks().cars[0].probe_count.get() == 0 {
        return Err(FatalError {
            code: rc2k_core::drive::race_ticks::NO_CAR_FATAL,
            message: "",
        }
        .into());
    }
    if race.shared.switches.get().levers_on.get() == 0 {
        return crate::drive::simple_car_step::simple_car_step(race, o.input, o.scene, o.net);
    }
    race.shared
        .vars
        .get()
        .race_ticks
        .set(race.shared.vars.get().race_ticks.get().wrapping_add(1));
    if !rc2k_core::drive::race_ticks::tick_runs(
        race.shared.vars.get().race_ticks.get(),
        race.shared.view.get().after_credits.get(),
    ) {
        return Ok(());
    }
    crate::drive::replay_step::replay_step(o, race)
}
