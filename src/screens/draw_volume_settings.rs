//! The sound page's volumes: the setting's name and eight rows, each a back sprite, a bar
//! (the volume as 0x40ths, percent `v * 100 / 0x40`), a knob and two strings. A volume whose
//! `v * 100` wraps to 0 raises fatal error 0x80701010.

use crate::errors::Refused;
use crate::owners::Owners;
use crate::text::draw_text::Canvas;

pub fn draw_volume_settings(o: Owners<'static>, canvas: &mut dyn Canvas) -> Result<(), Refused> {
    let menus = o.menus;
    let options = &menus.options;
    let levels = &o.audio.levels;
    // The logic is in `rc2k_core::screens::settings_pages::volume`.

    let id = o.sound.menu.options.race_music_on.get().wrapping_add(0x56D);
    // The volumes: the general volume, then `options.menu_music_volume` to
    // `audio.levels.engine_volume`.
    let volumes: [u32; 8] = [
        levels.general_volume.get(),
        options.menu_music_volume.get(),
        options.race_music_level.get(),
        options.menu_effects_volume.get(),
        levels.race_effects_volume.get(),
        options.menu_voices_volume.get(),
        levels.codriver_voice_volume.get(),
        levels.engine_volume.get(),
    ];
    rc2k_core::screens::settings_pages::volume(
        &mut crate::screens::game_screen::SettingsScreen::new(canvas, o),
        id,
        volumes,
    )
}
