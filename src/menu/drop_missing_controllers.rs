//! `menu::drop_missing_controllers` (0x4748B6): a car whose number at +0x2C is above
//! `dword_4F78E0` gets 0 there.
//!
//! In:  nothing
//! Out: every register preserved
//! For each of the four players, compared unsigned. A reading of what the two are:
//! `dword_4F78E0` is counted up by the function that lists the game controllers
//! (`sub_408238`, at most 0x10), so +0x2C is the player's controller, 0 being none.
//! Callers: `sub_47486C` (before the main menu) and `saves::load_savegame`. Refuses a missing
//! player. Rust callers run its `entry` (`routes.rs`); the registers above are the original's, for
//! reference.

use crate::errors::BadInput;
use crate::input::store::DEVICES;
use crate::menu::player_names::cars;

// ---- Rust version ----

pub fn drop_missing_controllers() -> Result<(), BadInput> {
    // Converted: `rc2k_core::menu::steps::kept_controller`.
    let most = DEVICES.controller_count.get();
    for car in cars() {
        let c = car.get().controller;
        let kept = rc2k_core::menu::steps::kept_controller(c, most);
        if kept != c {
            car.update(|p| p.controller = kept);
        }
    }
    Ok(())
}
