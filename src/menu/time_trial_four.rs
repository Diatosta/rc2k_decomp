//! `menu::time_trial_four` (0x48B90F): the time trial for four players in turn (event mode
//! 0x17). `g_eventMode` 3, `menu::begin_event`, `clear_players_284`,
//! `take_player1_units`; `dword_755984` 4, `dword_755988` 0x17, `dword_755954` 3,
//! `dword_75555D` 0 (no other variables cleared); the four car choices (player 0's list
//! made again before its car is taken; left, it ends at once). Then each stage as
//! `menu::time_trial_three`'s (`rc2k_core::menu::rounds::time_trial_rounds`) with four races and
//! these differences: no current player 0 before `menu::ready_players`, the service's yes
//! ends it, no stage record put in edi, and before the review `menu::show_in_game_options`
//! where the others hide them. The end: `rc2k_core::menu::rounds::end`.
//!
//! Reached through the time trial's start menu. Refuses a missing player before anything, and what
//! the callees refuse when they are reached.

use crate::errors::Refused;
use crate::menu::flow_game::Game;
use rc2k_core::menu::rounds;

pub fn time_trial_four(menus: &crate::menu::owner::Menus) -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::rounds::time_trial_four`.
    Game::new(menus).run(rounds::time_trial_four)
}
