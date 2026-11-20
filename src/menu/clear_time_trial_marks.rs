//! `menu::clear_time_trial_marks`: the +0x40 of each of the chosen stage's ten
//! best times set to 0 (the mark of a new entry, set by `menu::enter_time_trial_best`).
//!
//! Refuses (before anything) what `event_flow::stage_times` refuses.

use crate::errors::Refused;

pub(super) fn clear_time_trial_marks(
    menus: &crate::menu::owner::Menus,
    system: &crate::system::owner::System,
    tables: &crate::saves::best_times::BestTables,
) -> Result<(), Refused> {
    Ok(crate::menu::event_flow::with_stage_times(
        menus,
        system,
        tables,
        rc2k_core::saves::entries::clear_marks,
    )?)
}
