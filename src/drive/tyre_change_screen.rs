//! The tyre-change screen while a tyre change is under way (the first block's +0x26C not 0 or
//! -1): `drive::tyre_screen` for the mode `split`, car 0.

use crate::errors::Refused;
use crate::race::owner::Race;

pub fn tyre_change_screen(
    race: &Race,
    input: &crate::input::owner::Input,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    if !rc2k_core::drive::race_ticks::tyre_change_shown(
        crate::race::frame_view::car(0)?.tyre_change_state.get(),
    ) {
        return Ok(());
    }
    let shared = &race.shared;
    crate::drive::tyre_screen::tyre_screen(race, shared.vars.get().split.get(), 0, input, system)
}
