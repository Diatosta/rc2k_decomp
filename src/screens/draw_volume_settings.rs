//! `screens::draw_volume_settings` (0x476A92): the sound page's volumes: the setting's name
//! (string 0x56D + `dword_5180F4`, kept in `seconds`) at (0x1EA, 0x1B3), and eight rows from
//! y 0xDA every 0x1C: a back sprite 0x19 at x 0x222, the bar (`gfx::blit_bar` at x 0x221,
//! 0x76 wide, sprites 0x18 / 0x87, the volume `g_masterVolumeShown` / `dword_755688` ..
//! `dword_7556A0` as 0x40ths: percent v * 100 / 0x40), the knob sprite 0x1B at (0x1E6, y - 1)
//! and strings 0x547 / 0x548 at x 0x203 / 0x2B7, y + 3, in the font at 0x51E3C4.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! In a menu loop's draw list (0x758C8E). A volume whose v * 100 wraps to 0 is the original's fatal
//! 0x80701010 (`loc_4D9612`), raised from Rust where the original does. The sprites' palettes are
//! ebx as the original leaves it. The menus and draw lists run its `entry` (`menu::table_calls`);
//! the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::menu::option_work_store::OPTION_WORK;
use crate::text::draw_text::Canvas;

pub unsafe fn draw_volume_settings(canvas: &mut dyn Canvas) -> Result<(), Refused> {
    // Converted: `rc2k_core::screens::settings_pages::volume`.

    let id = crate::sound::store::SOUND_OPTIONS
        .name_switch
        .get()
        .wrapping_add(0x56D);
    // g_masterVolumeShown, then dword_755688 .. dword_7556A0.
    let volumes: [u32; 8] = [
        OPTION_WORK.master_volume_shown.get(),
        OPTION_WORK.music_level.get(),
        OPTION_WORK.other_volume.get(),
        OPTION_WORK.menu_sounds_c.get(),
        OPTION_WORK.menu_sounds_a.get(),
        OPTION_WORK.speech_level.get(),
        OPTION_WORK.sounds_a.get(),
        OPTION_WORK.sounds_b.get(),
    ];
    rc2k_core::screens::settings_pages::volume(
        &mut crate::screens::game_screen::SettingsScreen::new(canvas),
        id,
        volumes,
    )
}
