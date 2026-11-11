//! The left / right items of the menus' pages: each steps one setting a direction
//! (`forward` is the right arrow). In the original each direction was a function of its
//! own (`next_view` 0x467B45, `previous_view` 0x467B69, `next_car` 0x469AFD, ..).

use crate::errors::Refused;
use crate::menu::car_buttons::{Which, call_in_game, press};
use crate::menu::option_steps::{StartPick, step_start_pick};
use crate::menu::owner::Menus;
use crate::menu::owner::Tables;
use crate::menu::player_settings::current;
use rc2k_core::menu::player::{Setting, stepped};

/// The current player's `setting` (nationality, view, second view, race number) one step on
/// or back (see `rc2k_core::menu::player::stepped`).
pub fn player_setting(menus: &Menus, setting: Setting, forward: bool) -> Result<(), Refused> {
    current(menus)?.update(|player| {
        let value = player.setting(setting);
        *value = stepped(setting, *value, forward);
    });
    Ok(())
}

/// An options page's colour pick one step on or back.
pub fn colour(tables: &Tables, colour: StartPick, forward: bool) -> Result<(), Refused> {
    step_start_pick(tables, colour, forward);
    Ok(())
}

/// A button of the car specification (the car, its page, the slideshow's car or picture).
pub fn car_button(
    menus: &Menus,
    button: Which,
    sound: &crate::sound::owner::Sound,
) -> Result<(), Refused> {
    press(menus, button, &mut |call| call_in_game(menus, call, sound))
}

/// The saved games' list one page on (`menu::next_savegame_page`, 0x46D407): adds 1 to
/// `dword_75523C`, and above 9 goes back to 0.
pub fn next_savegame_page(tables: &Tables) {
    // The logic is in `rc2k_core::menu::steps::savegame_page`.
    let page = tables.pages.save_page.get();
    tables
        .pages
        .save_page
        .set(rc2k_core::menu::steps::savegame_page(page, true));
}
