//! `menu::time_trial_split` (0x48AFD0): the time trial for two players sharing the screen
//! (event mode 4). `g_eventMode` 3, `menu::begin_event`, `clear_players_284`,
//! `take_player1_units`; `dword_755984` 2, `dword_755988` 4, `dword_755954` 3,
//! `dword_75555D` 0, `dword_7587B8`, `dword_7587BC`, `dword_7587A8`, `dword_7587AC` 0; the
//! two car choices (left, it ends at once). Then each stage: the current player 0,
//! `menu::ready_players`, the repairs of players 0 and 1, players 2 and 3's times (+0x88)
//! 0x36F268 and 0x36F650, the stage choice (`menu::time_trial_select_stage_loop`; a yes or
//! `dword_75555D` ends it), `dword_755938` 1, the service (`menu::time_trial_service`; its
//! yes is not looked at), the race (eax 0, ebx 3); unless both results (`dword_75B028`,
//! `dword_75B098`) are -1: the results page (`menu::time_trial_results_loop`; a yes ends
//! it), the stage's record in edi, both players' figures kept, `dword_755920` 0, the marks
//! cleared, each player's time entered when it qualifies (`rc2k_core::menu::rounds::qualify`); the best
//! times page when one did (a yes ends it); the review (`rc2k_core::menu::rounds::review`, eax 2; its yes
//! not looked at). The end: `rc2k_core::menu::rounds::end`.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Reached through the time trial's start menu. Refuses a missing player before anything, and
//! what the callees and the stage's record (`Hand::Stage`) refuse when they are reached.
//! Hook removed 2026-10-22 (typed tables): the start menus run its `entry`
//! (`menu::table_calls`).

use crate::errors::Refused;
use crate::menu::event_flow::*;
use crate::menu::flow_game::Game;
use crate::menu::screen_loop::Carried;
use rc2k_core::menu::rounds;

pub unsafe fn time_trial_split(c: &Carried) -> Result<(), Refused> {
    players_there(4)?;
    // Converted: `rc2k_core::menu::rounds::time_trial_split`.
    rounds::time_trial_split(&mut Game::new(c))
}
