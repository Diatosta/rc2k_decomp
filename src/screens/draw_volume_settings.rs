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
    // The volumes: the master volume shown, then `options.music_level` to
    // `audio.levels.sounds_b`.
    let volumes: [u32; 8] = [
        levels.master_volume_shown.get(),
        options.music_level.get(),
        options.race_music_level.get(),
        options.menu_sounds_level.get(),
        levels.menu_sounds_a.get(),
        options.speech_level.get(),
        levels.sounds_a.get(),
        levels.sounds_b.get(),
    ];
    rc2k_core::screens::settings_pages::volume(
        &mut crate::screens::game_screen::SettingsScreen::new(canvas, o),
        id,
        volumes,
    )
}
