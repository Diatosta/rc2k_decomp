//! `menu::time_trial_one` (0x48AE39): the time trial for one player (event mode 0x14).
//! `menu::clear_players_284`, `take_player1_units`; `dword_755984` 1, `dword_755988`
//! 0x14, `dword_755954` 3, `dword_75555D` 0, `g_eventMode` 3; `menu::begin_event`; the
//! car choice (`rc2k_core::menu::rounds::choose_cars`, the time trial's list; left, it ends at once). Then
//! each stage: the stage choice (`menu::time_trial_select_stage_loop`; a yes or
//! `dword_75555D` ends it), `begin_event`, `dword_755938` 1, the service
//! (`menu::time_trial_service`; a yes ends it), the race (`race::single_rally_race`, eax
//! 0, ebx 3); unless its result is -1: the current player 0, the result page
//! (`menu::stage_result_loop`; a yes ends it), the stage's record in edi
//! (the stage's record (`Hand::Stage`)), `menu::keep_stage_best`, `dword_755918` 0,
//! `menu::time_trial_qualifies`, and when it qualifies `clear_time_trial_marks`,
//! `enter_time_trial_best` and the best times page (`time_trial_best_times_loop`; a yes
//! ends it); the review (`rc2k_core::menu::rounds::review`, eax 1, a yes asked). The end: `rc2k_core::menu::rounds::end`.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Reached through the time trial's start menu (0x48BCDC's items). Refuses a missing player before
//! anything, and what the callees and the stage's record (`Hand::Stage`) refuse when they are
//! reached.

use crate::errors::Refused;
use crate::menu::flow_game::Game;
use rc2k_core::menu::rounds;

pub fn time_trial_one() -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::rounds::time_trial_one`.
    Game::new().run(rounds::time_trial_one)
}
