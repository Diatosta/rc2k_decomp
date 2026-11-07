//! `menu::single_rally_split` (0x4824C1): the single rally for two players sharing the
//! screen (event mode 5). `menu::begin_event`, `clear_players_284`, `take_player1_units`;
//! `dword_755984` 2, `dword_755988` 5, `dword_75555D` 0; `clear_event_state`; the two car
//! choices (`rc2k_core::menu::rounds::choose_cars`; one left ends it at once). Then each stage: the current
//! player 0, `menu::ready_players`, the four repairs, the stage choice
//! (`menu::select_stage_loop`; a yes or `dword_75555D` ends it); player 0 taking part
//! (+0xD0 1), the repairs with ecx 0 and 1, the race (`race::single_rally_race`, eax 0,
//! ebx 0); unless both results (`dword_75B028`, `dword_75B098`) are -1 the results page
//! (`menu::results_48584b_loop`; a yes ends it); the review (`rc2k_core::menu::rounds::review`, eax 2; no
//! yes asked). The end: `rc2k_core::menu::rounds::end`.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Reached through the start menu's table. Refuses a missing player before anything, and what the
//! callees refuse when they are reached.

use crate::errors::Refused;
use crate::menu::flow_game::Game;
use rc2k_core::menu::rounds;

pub fn single_rally_split(menus: &crate::menu::owner::Menus) -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::rounds::single_rally_split`.
    Game::new(menus).run(rounds::single_rally_split)
}
