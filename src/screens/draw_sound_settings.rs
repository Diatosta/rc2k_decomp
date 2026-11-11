//! The sound page: items 4 .. 6 (3D sound, EAX, A3D 2) are choosable only when the card has
//! them, and the six settings' names (quality, channels, DS3D, EAX, A3D, Dolby: a string
//! picked by each value) are drawn at x 0x1D1 from y 0x10B every 0x1B in the item font.
//!
//! In a menu loop's draw list. Refuses what `menu::enable_item` refuses (no current menu),
//! before anything.

use crate::audio::owner::audio;
use crate::errors::Refused;
use crate::menu::disable_item::disable_item;
use crate::menu::enable_item::enable_item;
use crate::menu::owner::Menus;
use crate::session::DwordLike;
use crate::text::draw_text::Canvas;

/// (the setting, its first string).
fn names(menus: &Menus) -> [(&dyn DwordLike, u32); 6] {
    let work = &menus.screens.option_work;
    [
        (&work.audio_quality, 0x54B),
        (&work.channels, 0x554),
        (&work.ds3d, 0x56D),
        (&work.eax, 0x56D),
        (&work.a3d, 0x56D),
        (&work.dolby, 0x56D),
    ]
}

pub fn draw_sound_settings(menus: &Menus, canvas: &mut dyn Canvas) -> Result<(), Refused> {
    // The logic is in `rc2k_core::screens::settings_pages::{sound_items, sound}`.
    use crate::screens::game_screen::SettingsScreen;
    // The switches, by their items 4, 5 and 6: 3D, EAX and A3D2 there.
    let options = audio().state.audio_options.get();
    let [a, b, c] = [options.has_3d, options.has_eax, options.has_a3d2].map(|switch| switch == 1);
    for (item, on) in rc2k_core::screens::settings_pages::sound_items(a, b, c) {
        if on {
            enable_item(&menus.screens, item)?;
        } else {
            disable_item(&menus.screens, item)?;
        }
    }
    let ids = names(menus).map(|(setting, first)| setting.get().wrapping_add(first));
    rc2k_core::screens::settings_pages::sound(&mut SettingsScreen::new(canvas), ids)?;
    Ok(())
}
