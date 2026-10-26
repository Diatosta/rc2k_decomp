//! `menu::single_rally_three` (0x4828B0): the single rally for three players in turn
//! (event mode 8). `menu::begin_event`, `clear_players_284`, `take_player1_units`,
//! `clear_event_state`; `dword_758714` (the rounds) 0, `dword_755984` 3, `dword_755988`
//! 8, `dword_75555D` 0; the three car choices (one left ends it at once). Then each
//! round: the current player 0, `menu::ready_players`, the four repairs, the fourth
//! player's time (+0x88) 0x36F650, the stage choice (`menu::select_stage_loop`; a yes or
//! `dword_75555D` ends it); the three races (`rc2k_core::menu::rounds::race_series`, ebx 0, each car
//! repaired first; a race left, -2, goes to the review); unless all results are -1 the
//! results page (`menu::results_482e9b_loop`; a yes ends it) and from the second round
//! on the overall results (`menu::screen_483163_loop`; a yes ends it); the review
//! (`rc2k_core::menu::rounds::review`, eax 3, a yes asked); `dword_758714` + 1. The end: `rc2k_core::menu::rounds::end`.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Reached through the start menu's table. Refuses a missing player before anything, and what the
//! callees refuse when they are reached.

use crate::errors::Refused;
use crate::menu::event_flow::*;
use crate::menu::flow_game::Game;
use crate::menu::screen_loop::Carried;
use rc2k_core::menu::rounds;

pub fn single_rally_three(c: &Carried) -> Result<(), Refused> {
    players_there(4)?;
    // Converted: `rc2k_core::menu::rounds::single_rally_three`.
    Game::new(c).run(rounds::single_rally_three)
}
