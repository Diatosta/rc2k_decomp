//! The lobby's car item: runs the car screen from the player's car; unless left with Escape, the
//! chosen car and its colours go into the player's record. The background is drawn again after.

use crate::errors::Refused;
use crate::menu::screen_loop::{Entry, run_screen};
use crate::net::store::NetShared;
use crate::owners::Owners;
use rc2k_core::net::record;

/// The car screen is handed the list of cars; the
/// car it starts from its set-up finds itself (the current player's record,
/// `menu::car_choice::find_player_car`).
pub fn choose_car(o: Owners<'static>, vars: &NetShared) -> Result<(), Refused> {
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
        vars.own
            .head
            .set(record::CAR, play.players.players[0].get().car_number as u8);
        vars.own
            .head
            .set(record::COLOURS, play.players.players[0].get().gearing as u8);
    }
    crate::net::show_background::show_background(o)?;
    Ok(())
}
