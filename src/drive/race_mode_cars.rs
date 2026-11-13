//! By the race's mode `run_kind`: 2 runs `drive::start_cars_behind`; the others nothing.
//!
//! Refuses what `start_cars_behind` refuses.

use crate::errors::Refused;
use crate::race::owner::Race;

/// `drive::start_cars_behind` in mode 2.
pub fn race_mode_cars(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    if race.shared.vars.get().run_kind.get() == 2 {
        { crate::drive::start_cars_behind::start_cars_behind(race, scene) }?;
    }
    Ok(())
}
