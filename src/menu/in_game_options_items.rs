//! The in-game options menu's (`menu::in_game_options_loop`) items 3 .. 6, hidden and shown by
//! `menu::hide_in_game_options` and `menu::show_in_game_options`.
use crate::menu::owner::Menus;
use crate::menu::owner::Tables;
use rc2k_core::menu::items::{
    HIDDEN, IN_GAME_OPTIONS, Look, hidden_in_game_options, shown_in_game_option,
};

/// Item `n` given its look: its string, its state (skip mark) and what is behind its text.
pub fn set(tables: &Tables, n: u32, look: Look) -> Result<(), crate::errors::BadInput> {
    let item = tables
        .menus
        .in_game_options
        .id()
        .items(&tables.menus)
        .get(n as usize)
        .ok_or(crate::errors::BadInput("an item past the in-game options"))?;
    item.update(|state| {
        state.string = look.string;
        state.skip_mark = look.state;
        state.behind = look.behind;
    });
    Ok(())
}

/// `menu::hide_in_game_options`: for `from` 1, 2, 3 the items 3 + `from` .. 6 are hidden (no
/// string, passed over, nothing behind); anything else nothing.
pub fn hide_in_game_options(menus: &Menus, from: u32) -> Result<(), crate::errors::BadInput> {
    let tables = &menus.tables;
    for n in hidden_in_game_options(from) {
        set(tables, n, HIDDEN)?;
    }
    Ok(())
}

/// `menu::show_in_game_options`: items 3 .. 6 with their strings 0x171 .. 0x174, choosable.
pub fn show_in_game_options(menus: &Menus) -> Result<(), crate::errors::BadInput> {
    let tables = &menus.tables;
    for n in IN_GAME_OPTIONS {
        set(tables, n, shown_in_game_option(n))?;
    }
    Ok(())
}

/// The items of the cars' specifications hidden, as for any number of players that has none
/// (the network lobby has its own car button).
pub fn hide_car_specs(menus: &Menus) -> Result<(), crate::errors::BadInput> {
    for n in IN_GAME_OPTIONS {
        set(&menus.tables, n, HIDDEN)?;
    }
    Ok(())
}
