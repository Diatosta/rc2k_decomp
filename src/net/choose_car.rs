//! `net::choose_car` (0x4182FA): the lobby's car item: the current player 0, the car screen
//! (`menu::select_car_loop` on `net::car_list`'s list, from player 1's car, the Escape flag
//! `escape_pressed` 0 first); unless it was left with Escape, the car taken
//! (`menu::take_shown_car`) and this player's record given it (+1 the car, +2 its colours);
//! then the background again (`net::show_background`).
//!
//! Caller: `net::lobby_loop`.

use crate::errors::Refused;
use crate::menu::screen_loop::{Entry, run_screen};
use crate::net::store::NetVars;
use crate::net::store::PlayerRecord;
use crate::owners::Owners;

/// The car screen is handed the list of cars; the
/// car it starts from its set-up finds itself (the current player's record,
/// `menu::car_choice::find_player_car`).
pub fn choose_car(o: Owners<'_>, vars: &NetVars) -> Result<(), Refused> {
    let menus = o.menus;
    let play = &menus.play;
    // The current player of the event, and whether the screen was left with Escape.
    let escape_pressed = &menus.screens.draw_work.escape_pressed;
    menus.screens.event.current_player.set(0);
    let cars = crate::net::car_list::car_list(&menus.tables)?;
    escape_pressed.set(false);
    let entry = Entry::with_cars(o, Some(cars));
    { run_screen(&crate::menu::select_car_loop::SCREEN, &entry) }?;
    if !escape_pressed.is_set() {
        crate::menu::take_shown_car::take_shown_car(menus, Some(cars))?;
        vars.own.head.set(
            PlayerRecord::CAR,
            play.players.players[0].get().car_number as u8,
        );
        vars.own.head.set(
            PlayerRecord::COLOURS,
            play.players.players[0].get().gearing as u8,
        );
    }
    crate::net::show_background::show_background(o.gfx, vars)?;
    Ok(())
}
