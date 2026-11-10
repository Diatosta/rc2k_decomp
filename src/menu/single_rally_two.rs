//! `menu::single_rally_two` (0x4826A2): the single rally for two players in turn (event
//! mode 8). `menu::begin_event`, `clear_hud_bits`, `take_player1_units`;
//! `dword_755984` 2, `dword_755988` 8, `dword_75555D` 0; `clear_event_state`; the two car
//! choices (one left ends it at once). Then each stage: the current player 0,
//! `menu::ready_players`, the four repairs, the stage choice (`menu::select_stage_loop`; a
//! yes or `dword_75555D` ends it); the two races (`rc2k_core::menu::rounds::race_series`, ebx 0, each car
//! repaired first; a race left, -2, goes to the review); unless all results are -1 the
//! results page (`menu::single_rally_two_results_loop`; a yes ends it); the review (`rc2k_core::menu::rounds::review`,
//! eax 2, a yes asked). The end: `rc2k_core::menu::rounds::end`.
//!
//! Reached through the start menu's table. Refuses a missing player before anything, and what the
//! callees refuse when they are reached.

use crate::errors::Refused;
use crate::menu::flow_game::Game;
use rc2k_core::menu::rounds;

pub fn single_rally_two(menus: &crate::menu::owner::Menus) -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::rounds::single_rally_two`.
    Game::new(menus).run(rounds::single_rally_two)
}
