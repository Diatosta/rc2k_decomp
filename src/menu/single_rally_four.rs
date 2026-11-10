//! `menu::single_rally_four` (0x482B7A): the single rally for four players in turn (event
//! mode 8). `menu::begin_event`, `clear_hud_bits`, `take_player1_units`,
//! `clear_event_state`; `dword_755984` 4, `dword_755988` 8 (neither `dword_75555D` nor
//! `dword_758714` set to 0, unlike the three players'); the four car choices (one left
//! ends it at once). Then each round as `menu::single_rally_three`'s
//! (`rc2k_core::menu::rounds::rally_rounds`) but without the current player set to 0 before
//! `menu::ready_players` and without the fourth player's time: four races, eax 4 for the
//! review. The end: `rc2k_core::menu::rounds::end`.
//!
//! Reached through the start menu's table. Refuses a missing player before anything, and what the
//! callees refuse when they are reached.

use crate::errors::Refused;
use crate::menu::flow_game::Game;
use rc2k_core::menu::rounds;

pub fn single_rally_four(menus: &crate::menu::owner::Menus) -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::rounds::single_rally_four`.
    Game::new(menus).run(rounds::single_rally_four)
}
