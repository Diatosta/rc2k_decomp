//! `menu::drop_missing_controllers`: a car whose number at +0x2C is above
//! the controller count gets 0 there.
//!
//! For each of the four players, compared unsigned. A reading of what the two are: the controller
//! count is counted up by the function that lists the game controllers (`input::joystick_found`, at
//! most 0x10), so +0x2C is the player's controller, 0 being none. Refuses a missing player.

use crate::errors::BadInput;
use crate::input::owner::Input;
use crate::menu::owner::Menus;
use crate::menu::player_names::cars;

pub fn drop_missing_controllers(menus: &Menus, input: &Input) -> Result<(), BadInput> {
    let play = &menus.play;
    let most = input.devices.controller_count.get();
    for car in cars(play) {
        let c = car.get().controller;
        let kept = rc2k_core::menu::steps::kept_controller(c, most);
        if kept != c {
            car.update(|p| p.controller = kept);
        }
    }
    Ok(())
}
