//! The keys on a redefine keys page: the current player's keys, places 0 to 7 and 9 to 11
//! (place 8 is not drawn), each as the key's name, but at the place being redefined the cursor
//! while the text cursor's blink is on, else nothing. The player's keys are drawn, not the page's
//! work copy: a key shows when the redefining ends. Refuses no current player and what the key's
//! name refuses.

use crate::errors::Refused;
use crate::input::append_key_name::append_key_name;
use crate::menu::player_settings::current;
use crate::text::draw_text::Canvas;

// ---- game data ----

// the cells text buffer
// up to the stage heading buffer

/// Returns whether the last text drawn was too wide for its box.
pub fn draw_redefined_keys(
    o: crate::owners::Owners<'static>,
    canvas: &mut dyn Canvas,
) -> Result<bool, Refused> {
    let menus = o.menus;
    let screens = &menus.screens;
    let keys = current(menus)?.get().keys;
    let redefined: u32 = screens.entry.redefined_key_place.get();
    // The logic is in `rc2k_core::screens::controls::redefined_keys`; the names are made first
    // (none at the place being redefined).
    use rc2k_core::screens::controls::KEY_PLACES;
    let row = KEY_PLACES.iter().position(|&p| p as u32 == redefined);
    let mut names: [Vec<u8>; 11] = Default::default();
    for (n, name) in names.iter_mut().enumerate() {
        if row != Some(n) {
            *name = append_key_name(
                crate::strings::append_string_by_id::StringOwners::of(o),
                keys[KEY_PLACES[n]] as u32,
            )?
            .0;
        }
    }
    rc2k_core::screens::controls::redefined_keys(
        &mut crate::screens::game_screen::ControlsScreen::new(canvas, o),
        &names,
        row,
        screens.draw_work.cursor_on.is_set(),
    )
}
