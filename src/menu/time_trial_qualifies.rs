//! `menu::time_trial_qualifies` (0x48C7CE): whether the current player's time makes the
//! chosen stage's best times: `dword_755918` 0; `dword_755914` the player's +0x88 (the
//! time); when that is not above (signed) the tenth entry's +0x14, `dword_755918` and
//! `dword_755920` 1.
//!
//! Callers: the time trial's events, after the results. Refuses (before anything) a current player
//! past the four or missing, a stage kind past the table or a null pointer to the stage's records
//! or times (`event_flow::stage_times`).

use crate::errors::Refused;
use crate::menu::event_flow::*;
use crate::session::Dword;

pub const QUALIFIES: &Dword = &crate::menu::owner::MENUS.screens.event.time_qualifies; // dword_755918
pub const TIME: &Dword = &crate::menu::owner::MENUS.screens.event.player_time; // dword_755914
pub const ANY_QUALIFIED: &Dword = &crate::menu::owner::MENUS.screens.event.any_time_qualified; // dword_755920

pub fn time_trial_qualifies() -> Result<(), Refused> {
    stage_times()?;
    let p = current_player()?;
    set(QUALIFIES, 0);
    let time = p.get().time;
    set(TIME, time);
    // Converted: `rc2k_core::menu::bests::makes_time`.
    let best = with_stage_times(|ten| ten[9].head.key)?;
    if rc2k_core::menu::bests::makes_time(time, best) {
        set(QUALIFIES, 1);
        set(ANY_QUALIFIED, 1);
    }
    Ok(())
}
