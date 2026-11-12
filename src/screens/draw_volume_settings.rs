//! `screens::draw_volume_settings` (0x476A92): the sound page's volumes: the setting's name
//! (string 0x56D + the volume page flag, kept in `seconds`) at (0x1EA, 0x1B3), and eight rows from
//! y 0xDA every 0x1C: a back sprite 0x19 at x 0x222, the bar (`gfx::blit_bar` at x 0x221,
//! 0x76 wide, sprites 0x18 / 0x87, the volume `g_masterVolumeShown` / `option_work.music_level` ..
//! `audio.levels.sounds_b` as 0x40ths: percent v * 100 / 0x40), the knob sprite 0x1B at (0x1E6, y - 1)
//! and strings 0x547 / 0x548 at x 0x203 / 0x2B7, y + 3, in the font at 0x51E3C4.
//!
//!
//! In a menu loop's draw list (0x758C8E). A volume whose v * 100 wraps to 0 is the original's fatal
//! 0x80701010 (the shared tail of that routine), raised from Rust where the original does. The sprites' palettes are
//! ebx as the original leaves it. The menus and draw lists run its `entry` (`menu::table_calls`).

use crate::errors::Refused;
use crate::owners::Owners;
use crate::text::draw_text::Canvas;

pub fn draw_volume_settings(o: Owners<'static>, canvas: &mut dyn Canvas) -> Result<(), Refused> {
    let menus = o.menus;
    let screens = &menus.screens;
    let levels = &o.audio.levels;
    // The logic is in `rc2k_core::screens::settings_pages::volume`.

    let id = o.sound.menu.options.race_music_on.get().wrapping_add(0x56D);
    // g_masterVolumeShown, then `option_work.music_level` .. `option_work.sounds_b`.
    let volumes: [u32; 8] = [
        levels.master_volume_shown.get(),
        screens.option_work.music_level.get(),
        screens.option_work.other_volume.get(),
        screens.option_work.menu_sounds_c.get(),
        levels.menu_sounds_a.get(),
        screens.option_work.speech_level.get(),
        levels.sounds_a.get(),
        levels.sounds_b.get(),
    ];
    rc2k_core::screens::settings_pages::volume(
        &mut crate::screens::game_screen::SettingsScreen::new(canvas, o),
        id,
        volumes,
    )
}
