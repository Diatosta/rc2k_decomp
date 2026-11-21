//! `menu::rank_time_trial_best`: the tenth of the chosen stage's best times
//! moved up to its place: from the tenth to the second, each entry whose `key` (the time)
//! is below (unsigned) the one before swaps with it, field by field: all of them, the checkpoints
//! line and the 31 name bytes (through the kept player name, which keeps the last swap's upper
//! name). The pass does not stop at the first entry in order.
//!
//! Refuses (before anything) what `event_flow::stage_times` refuses.

use crate::errors::Refused;
use crate::menu::event_flow::*;

pub(super) fn rank_time_trial_best(
    menus: &crate::menu::owner::Menus,
    system: &crate::system::owner::System,
    tables: &crate::saves::best_times::BestTables,
) -> Result<(), Refused> {
    Ok(with_stage_times(menus, system, tables, |list| {
        crate::menu::standings::with_kept_name(&menus.play, |kept| {
            rc2k_core::saves::entries::rank_time_trial(list, kept);
        });
    })?)
}
