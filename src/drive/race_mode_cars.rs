//! `drive::race_mode_cars` (0x436758): by the race's mode `run_kind`: 2 runs
//! `drive::start_cars_behind`; 0, 1, 3, 4 and the others nothing (five separate `retn`s).
//!
//! Callers: race init's loop (`drive::race_cars_step`), `race::step_replay_cars`,
//! `drive::replay_step`. Refuses what
//! `start_cars_behind` refuses.

use crate::errors::Refused;
use crate::race::owner::Race;

/// `drive::start_cars_behind` in mode 2.
pub fn race_mode_cars(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    if race.shared.vars.run_kind.get() == 2 {
        { crate::drive::start_cars_behind::start_cars_behind(race, scene) }?;
    }
    Ok(())
}
