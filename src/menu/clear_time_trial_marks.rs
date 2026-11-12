//! `menu::clear_time_trial_marks` (0x48C85C): the +0x40 of each of the chosen stage's ten
//! best times set to 0 (the mark of a new entry, set by `menu::enter_time_trial_best`).
//!
//! Callers: the time trial's events. Refuses (before anything) what `event_flow::stage_times`
//! refuses.

use crate::errors::Refused;

pub fn clear_time_trial_marks(
    menus: &crate::menu::owner::Menus,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    // The logic is in `rc2k_core::saves::entries::clear_marks`.
    Ok(crate::menu::event_flow::with_stage_times(
        menus,
        system,
        rc2k_core::saves::entries::clear_marks,
    )?)
}
