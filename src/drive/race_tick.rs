//! One tick of the race. Fatal error 0x80600655 (no message) when the first car block's `probe_count`
//! is 0. Free roaming (`levers_on` 0): `drive::simple_car_step`. Else the tick count
//! `race_ticks` + 1 and `drive::replay_step` every tick, or with `after_credits` set every
//! second tick, and past 1 every fourth.

use crate::errors::{FatalError, Refused};
use crate::race::owner::Race;

pub fn race_tick(o: crate::owners::Owners<'static>, race: &Race) -> Result<(), Refused> {
    if race.car_blocks().cars[0].probe_count.get() == 0 {
        return Err(FatalError {
            code: rc2k_core::drive::race_ticks::NO_CAR_FATAL,
            message: "",
        }
        .into());
    }
    if race.shared.switches.get().levers_on.get() == 0 {
        crate::drive::simple_car_step::simple_car_step(race, o.input, o.scene, o.net)?;
        crate::race::blend::note_tick(race);
        return Ok(());
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
    crate::drive::replay_step::replay_step(o, race)?;
    // Only a tick that moved the cars (a credits race's skipped ticks did not) is a state to
    // blend from.
    crate::race::blend::note_tick(race);
    Ok(())
}
