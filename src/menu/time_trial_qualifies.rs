//! `menu::time_trial_qualifies`: whether the current player's time makes the chosen stage's best
//! times: `event.trial.qualifies` is set to 0; when the player's time (+0x88) is not above (signed)
//! the tenth entry's +0x14, `event.trial.qualifies` and `event.trial.any_qualified` become 1.
//!
//! Callers: the time trial's events, after the results. Refuses (before anything) a current player
//! past the four or missing, a stage kind past the table or a null pointer to the stage's records
//! or times (`event_flow::stage_times`).

use crate::errors::Refused;
use crate::menu::event_flow::*;
use crate::menu::owner::Menus;

pub fn time_trial_qualifies(
    menus: &Menus,
    system: &crate::system::owner::System,
    tables: &crate::saves::best_times::BestTables,
) -> Result<(), Refused> {
    stage_times(menus, system, tables)?;
    let p = current_player(menus)?;
    let event = &menus.screens.event;
    set(&event.trial.qualifies, 0);
    let time = p.get().time;
    let best = with_stage_times(menus, system, tables, |ten| ten[9].head.key)?;
    if rc2k_core::menu::bests::makes_time(time, best) {
        set(&event.trial.qualifies, 1);
        event.trial.any_qualified.set(true);
    }
    Ok(())
}
