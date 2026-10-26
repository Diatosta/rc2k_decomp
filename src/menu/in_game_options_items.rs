//! The in-game options menu's (`menu::in_game_options_loop`, `off_750724`) items 3 .. 6,
//! hidden and shown by `menu::hide_in_game_options` and `menu::show_in_game_options`
//! (IDA, 2026-10-15).

use crate::menu::item_at::{ITEM_LEN, MENU_HEAD_LEN};
use crate::menu::menus_store::MENUS;
use rc2k_core::menu::items::{
    HIDDEN, IN_GAME_OPTIONS, Look, hidden_in_game_options, shown_in_game_option,
};

pub fn item(n: u32) -> u32 {
    MENUS.in_game_options.addr() as u32 + MENU_HEAD_LEN + n * ITEM_LEN
}

/// Item `n` given its look: +0x14, +0x48, +0x1C, in the original's order.
pub fn set(n: u32, look: Look) -> Result<(), crate::errors::BadInput> {
    let at = crate::pointers::GamePtr::new(item(n));
    at.offset(0x14).write::<u32>(look.string, "a menu item")?;
    at.offset(0x48).write::<u32>(look.state, "a menu item")?;
    at.offset(0x1C).write::<u32>(look.behind, "a menu item")?;
    Ok(())
}

// Converted: `rc2k_core::menu::items` (`hidden_in_game_options`, `shown_in_game_option`).

/// `menu::hide_in_game_options`: eax 1, 2, 3: items 3 + eax .. 6 hidden (no string, passed
/// over, nothing behind); anything else nothing.
pub fn hide_in_game_options(from: u32) -> Result<(), crate::errors::BadInput> {
    for n in hidden_in_game_options(from) {
        set(n, HIDDEN)?;
    }
    Ok(())
}

/// `menu::show_in_game_options`: items 3 .. 6 with their strings 0x171 .. 0x174, choosable.
pub fn show_in_game_options() -> Result<(), crate::errors::BadInput> {
    for n in IN_GAME_OPTIONS {
        set(n, shown_in_game_option(n))?;
    }
    Ok(())
}
