//! The sound page: items 4 and 5 (3D sound, EAX) are choosable only when the card has
//! them, and the five settings' names (quality, channels, DS3D, EAX, Dolby: a string picked
//! by each value) are drawn in the item font. Refuses what `menu::enable_item` refuses (no current
//! menu).

use crate::errors::Refused;
use crate::memory::session::DwordLike;
use crate::menu::disable_item::disable_item;
use crate::menu::enable_item::enable_item;
use crate::menu::owner::Menus;
use crate::text::draw_text::Canvas;

/// (the setting, its first string).
fn names(menus: &Menus) -> [(&dyn DwordLike, u32); 5] {
    let work = &menus.options;
    [
        (&work.audio_quality, 0x54B),
        (&work.channels, 0x554),
        (&work.ds3d, 0x56D),
        (&work.eax, 0x56D),
        (&work.dolby, 0x56D),
    ]
}

pub fn draw_sound_settings(
    o: crate::owners::Owners<'static>,
    audio: &crate::audio::owner::Audio,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    let menus = o.menus;
    // The logic is in `rc2k_core::screens::settings_pages::{sound_items, sound}`.
    use crate::screens::game_screen::SettingsScreen;
    // The switches, by their items 4 and 5: 3D and EAX there.
    let options = audio.state.audio_options.get();
    let [has_3d, has_eax] = [options.has_3d, options.has_eax].map(|switch| switch == 1);
    for (item, on) in rc2k_core::screens::settings_pages::sound_items(has_3d, has_eax) {
        if on {
            enable_item(menus, item)?;
        } else {
            disable_item(menus, item)?;
        }
    }
    let ids = names(menus).map(|(setting, first)| setting.get().wrapping_add(first));
    rc2k_core::screens::settings_pages::sound(&mut SettingsScreen::new(canvas, o), ids)?;
    Ok(())
}
