//! The tyre-change screen while a tyre change is under way (the first block's `tyre_change_state` not 0 or
//! -1): `drive::tyre_screen` for the mode `split`, car 0.

use crate::errors::Refused;
use crate::race::owner::Race;

pub fn tyre_change_screen(
    race: &Race,
    input: &crate::input::owner::Input,
    system: &crate::system::owner::System,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    if !rc2k_core::drive::race_ticks::tyre_change_shown(
        race.frame.cars.car(0)?.tyre_change_state.get(),
    ) {
        return Ok(());
    }
    let shared = &race.shared;
    crate::drive::tyre_screen::tyre_screen(
        race,
        shared.vars.get().split.get(),
        0,
        input,
        system,
        scene,
    )
}
