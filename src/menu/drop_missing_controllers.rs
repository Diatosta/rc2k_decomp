//! `menu::drop_missing_controllers` (0x4748B6): a car whose number at +0x2C is above
//! `dword_4F78E0` gets 0 there.
//!
//! For each of the four players, compared unsigned. A reading of what the two are:
//! `dword_4F78E0` is counted up by the function that lists the game controllers
//! (`input::joystick_found`, at most 0x10), so +0x2C is the player's controller, 0 being none.
//! Callers: `menu::run_front_end` (before the main menu) and `saves::load_savegame`. Refuses a
//! missing player. Rust callers run its `entry` (`routes.rs`).

use crate::errors::BadInput;
use crate::input::owner::input;
use crate::menu::owner::Menus;
use crate::menu::player_names::cars;

pub fn drop_missing_controllers(menus: &Menus) -> Result<(), BadInput> {
    let play = &menus.play;
    // Converted: `rc2k_core::menu::steps::kept_controller`.
    let most = input().devices.controller_count.get();
    for car in cars(play) {
        let c = car.get().controller;
        let kept = rc2k_core::menu::steps::kept_controller(c, most);
        if kept != c {
            car.update(|p| p.controller = kept);
        }
    }
    Ok(())
}
