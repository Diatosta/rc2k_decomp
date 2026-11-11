//! `screens::draw_redefined_keys` (0x47CFA7): the keys on a redefine keys page.
//!
//!
//! The current player's (`event.current_player`) keys (`Player::keys`, +0x1C), places 0 .. 7 and 9 .. 11
//! (place 8 is not drawn), at x = 0x1CC, y = 0xAF and on every 0x1D, in the font at 0x51E8C4: each
//! key's name (`input::append_key_name` and a NUL into the cells text buffer, then `draw_text`), but at the
//! place being redefined (`g_redefinedKeyPlace`) the cursor, string 4, while the text cursor's
//! blink (`draw_work.cursor_on`) is 1, else nothing (the carry then that of `cmp `draw_work.cursor_on`, 1`). The
//! player's keys, not the page's work copy: a key shows when the redefining ends. In the draw lists
//! of the four pages (0x758DE6 ..). Refuses no current player and what the key's name refuses. The
//! menus and draw lists run its `entry` (`menu::table_calls`).

use crate::errors::Refused;
use crate::input::append_key_name::append_key_name;
use crate::menu::owner::Menus;
use crate::menu::player_settings::current;
use crate::text::draw_text::Canvas;

// ---- game data ----

// the cells text buffer
// up to the stage heading buffer

/// Returns the carry.
pub fn draw_redefined_keys(menus: &Menus, canvas: &mut dyn Canvas) -> Result<bool, Refused> {
    let screens = &menus.screens;
    let keys = current(menus)?.get().keys;
    let redefined: u32 = screens.option_work.redefined_key_place.get();
    // The logic is in `rc2k_core::screens::controls::redefined_keys`; the names made first
    // (none at the place being redefined, as the original).
    use rc2k_core::screens::controls::KEY_PLACES;
    let row = KEY_PLACES.iter().position(|&p| p as u32 == redefined);
    let mut names: [Vec<u8>; 11] = Default::default();
    for (n, name) in names.iter_mut().enumerate() {
        if row != Some(n) {
            *name = append_key_name(keys[KEY_PLACES[n]] as u32)?.0;
        }
    }
    rc2k_core::screens::controls::redefined_keys(
        &mut crate::screens::game_screen::ControlsScreen::new(canvas),
        &names,
        row,
        screens.draw_work.cursor_on.is_set(),
    )
}
