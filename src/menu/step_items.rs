//! The left / right items of the menus' pages: each steps one setting a direction (`forward` is
//! the right arrow).

use crate::errors::Refused;
use crate::menu::car_buttons::{Which, make_call, press};
use crate::menu::option_steps::{StartPick, step_start_pick};
use crate::menu::owner::Menus;
use crate::menu::owner::Tables;
use crate::menu::player_settings::current;
use crate::owners::Owners;
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
pub fn colour(tables: &Tables, colour: StartPick<'_>, forward: bool) -> Result<(), Refused> {
    step_start_pick(tables, colour, forward);
    Ok(())
}

/// A button of the car specification (the car, its page, the slideshow's car or picture).
pub fn car_button(o: Owners<'static>, button: Which) -> Result<(), Refused> {
    press(o.menus, button, &mut |call| make_call(o, call))
}

/// The saved games' list one page on (`menu::next_savegame_page`): adds 1 to
/// `MenuPages.save_page`, and above 9 goes back to 0.
pub fn next_savegame_page(tables: &Tables) {
    let page = tables.pages.save_page.get();
    tables
        .pages
        .save_page
        .set(rc2k_core::menu::steps::savegame_page(page, true));
}
