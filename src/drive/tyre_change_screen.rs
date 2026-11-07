//! `drive::tyre_change_screen` (0x445047): the tyre-change screen while a tyre change is
//! under way (the first block's +0x26C, `g_carBlockStore`, not 0 or -1):
//! `drive::tyre_screen` for the mode `dword_6238A4`, car 0.
//!
//! Caller: `race::draw_race_scene` (the race's frame, stream A's).

use crate::errors::Refused;
use crate::race::blocks_store::CAR_BLOCKS;
use crate::race::owner::Race;

/// The Rust version; every register kept.
// Converted: `rc2k_core::drive::race_ticks::tyre_change_shown`.
pub fn tyre_change_screen(
    race: &Race,
    input: &crate::input::owner::Input,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    if !rc2k_core::drive::race_ticks::tyre_change_shown(CAR_BLOCKS.cars[0].tyre_change_state.get())
    {
        return Ok(());
    }
    let shared = &race.shared;
    crate::drive::tyre_screen::tyre_screen(shared, shared.vars.split.get(), 0, input, system)
}
