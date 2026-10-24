//! `menu::single_rally` (0x482382): the single rally's event, from the player choice on.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! `menu::begin_event`, `clear_players_284`, `take_player1_units`; `dword_755984` 1,
//! `dword_755988` 0x18, `dword_75555D` 0; `clear_event_state`; the current player 0; the car
//! list in esi (`off_759DCA` with `dword_7554D8` 1, else `off_75A042`; `off_75A412` with
//! cheat bit 1); `menu::select_car_loop`, and with `dword_758806` 1 straight back (without
//! the end below); `take_shown_car`. Then each stage: `menu::select_stage_loop` (ended by
//! `g_yesNoAnswer` 1 or `dword_75555D` 1), the current player's +0xD0 1,
//! `service::repair_car`, the race `sub_4843EC` (eax 0, ebx 0); unless `dword_75B028` is -1
//! the current player 0 and `menu::stage_result_loop` (ended by a "yes"); eax 1,
//! `hide_in_game_options`, `menu::review_options_485f5e_loop` (ended by `dword_755A3C`,
//! `dword_755569` or `dword_75555D` 1), and the next. The end: `dword_755980` 1,
//! `dword_75555D`, `dword_755569`, `dword_755565` 0, `show_in_game_options`. Reached from
//! the start menu's table (0x748A8C). The registers each callee gets are the original's
//! (one set carried through, as `pusha` keeps them). Refuses a current player past four.
//! The screens and the race are called at the game's addresses (their hooks choose).
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::menu::flow_game::Game;
use crate::menu::screen_loop::Carried;

/// The Rust version. `c` the carried values (the registers the item gets).
pub unsafe fn single_rally(c: &Carried) -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::rounds::single_rally`.
    rc2k_core::menu::rounds::single_rally(&mut Game::new(c))
}
