//! `menu::time_trial_qualifies` (0x48C7CE): whether the current player's time makes the
//! chosen stage's best times: `dword_755918` 0; `dword_755914` the player's +0x88 (the
//! time); when that is not above (signed) the tenth entry's +0x14, `dword_755918` and
//! `dword_755920` 1.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Callers: the time trial's events, after the results. Refuses (before anything) a current player
//! past the four or missing, a stage kind past the table or a null pointer to the stage's records
//! or times (`event_flow::stage_times`).

use crate::errors::Refused;
use crate::menu::event_flow::*;
use crate::menu::event_store::EVENT_STATE;
use crate::session::Dword;

pub const QUALIFIES: &Dword = &EVENT_STATE.time_qualifies; // dword_755918
pub const TIME: &Dword = &EVENT_STATE.player_time; // dword_755914
pub const ANY_QUALIFIED: &Dword = &EVENT_STATE.any_time_qualified; // dword_755920
/// The tenth entry of the best times.
pub const TENTH: u32 = 9 * TIME_ENTRY;

pub fn time_trial_qualifies() -> Result<(), Refused> {
    let tenth = stage_times()? + TENTH;
    let p = current_player()?;
    set(QUALIFIES, 0);
    let time = dword(p + 0x88)?;
    set(TIME, time);
    // Converted: `rc2k_core::menu::bests::makes_time`.
    if rc2k_core::menu::bests::makes_time(time, dword(tenth + 0x14)?) {
        set(QUALIFIES, 1);
        set(ANY_QUALIFIED, 1);
    }
    Ok(())
}
