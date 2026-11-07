//! `net::choose_car` (0x4182FA): the lobby's car item: `dword_75598C` 0, the car screen
//! (`menu::select_car_loop` on `net::car_list`'s list, from player 1's car, the Escape flag
//! `dword_755A3C` 0 first); unless it was left with Escape, the car taken
//! (`menu::take_shown_car`) and this player's record given it (+1 the car, +2 its colours);
//! then the background again (`net::show_background`).
//!
//! In:  nothing
//! Out: nothing (the original keeps every register: `pusha` .. `popa`)
//! Caller: `net::lobby_loop`.

use crate::errors::Refused;
use crate::menu::owner::menus;
use crate::menu::screen_loop::{Entry, run_screen};
use crate::net::store::NetVars;
use crate::net::store::PlayerRecord;
use crate::session::Dword;

const CLEARED: &Dword = &crate::menu::owner::MENUS.screens.event.current_player; // dword_75598C
pub const ESCAPED: &Dword = &crate::menu::owner::MENUS.screens.draw_work.escaped; // dword_755A3C: the screen was left with Escape

/// The Rust version. The car screen is handed the list of cars (the original's esi); the
/// car it starts from its set-up finds itself (the current player's record,
/// `menu::car_choice::find_player_car`).
pub fn choose_car(vars: &NetVars) -> Result<(), Refused> {
    let menus = menus();
    let play = &menus.play;
    CLEARED.set(0);
    let cars = crate::net::car_list::car_list(&menus.tables)?;
    ESCAPED.set(0);
    let entry = Entry::with_cars(menus, Some(cars));
    { run_screen(&crate::menu::select_car_loop::SCREEN, &entry) }?;
    if ESCAPED.get() != 1 {
        crate::menu::take_shown_car::take_shown_car(Some(cars))?;
        vars.own.head.set(
            PlayerRecord::CAR,
            play.players.players[0].get().car_number as u8,
        );
        vars.own.head.set(
            PlayerRecord::COLOURS,
            play.players.players[0].get().gearing as u8,
        );
    }
    crate::net::show_background::show_background(vars)?;
    Ok(())
}
