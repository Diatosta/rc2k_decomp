//! The in-game options menu's (`menu::in_game_options_loop`, `off_750724`) items 3 .. 6,
//! hidden and shown by `menu::hide_in_game_options` and `menu::show_in_game_options`
//! (IDA, 2026-10-15).
use crate::menu::owner::menus;
use rc2k_core::menu::items::{
    HIDDEN, IN_GAME_OPTIONS, Look, hidden_in_game_options, shown_in_game_option,
};

/// Item `n` given its look: +0x14, +0x48, +0x1C, in the original's order.
pub fn set(n: u32, look: Look) -> Result<(), crate::errors::BadInput> {
    let item = menus()
        .tables
        .menus
        .in_game_options
        .items
        .get(n as usize)
        .ok_or(crate::errors::BadInput("an item past the in-game options"))?;
    item.string.set(look.string);
    item.skip_mark.set(look.state);
    item.behind.set(look.behind);
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
