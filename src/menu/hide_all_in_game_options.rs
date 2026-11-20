//! `menu::hide_all_in_game_options`: four items of the in-game options menu given an empty string,
//! the skip mark (`skip_mark`, passed over by the moves) and -0x57 as `behind` (a reading: the cars
//! on the front end's carousel at their start).
//!
//! They are the in-game options menu's items 3 .. 6 (`menu/in_game_options_items.rs`): this hides
//! all four, as `menu::hide_in_game_options` hides the last ones.
use crate::errors::Refused;
use crate::menu::in_game_options_items::set;
use crate::menu::owner::Menus;
use rc2k_core::menu::items::{HIDDEN, IN_GAME_OPTIONS};
pub(super) fn hide_all_in_game_options(menus: &Menus) -> Result<(), Refused> {
    let tables = &menus.tables;
    // The logic is in `rc2k_core::menu::items::HIDDEN` (the four items `hide_in_game_options`
    // hides, all of them).
    for n in IN_GAME_OPTIONS {
        set(tables, n, HIDDEN)?;
    }
    Ok(())
}
