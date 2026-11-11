//! `menu::hide_all_in_game_options` (0x46FC52): four entries of the in-game options menu's table (0x5C apart
//! from +0x118) given +0x14 0, +0x48 -0x3E4 and +0x1C -0x57 (a reading: the cars on the
//! front end's carousel at their start).
//!
//! They are the in-game options menu's items 3 .. 6 (`menu/in_game_options_items.rs`):
//! this hides all four, as `menu::hide_in_game_options` hides the last ones.
//! Callers: the event drivers.
use crate::errors::Refused;
use crate::menu::in_game_options_items::set;
use crate::menu::owner::Menus;
use rc2k_core::menu::items::{HIDDEN, IN_GAME_OPTIONS};
pub fn hide_all_in_game_options(menus: &Menus) -> Result<(), Refused> {
    let tables = &menus.tables;
    // Converted: `rc2k_core::menu::items::HIDDEN` (the four items `hide_in_game_options`
    // hides, all of them).
    for n in IN_GAME_OPTIONS {
        set(tables, n, HIDDEN)?;
    }
    Ok(())
}
