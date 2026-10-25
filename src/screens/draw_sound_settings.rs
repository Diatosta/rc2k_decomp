//! `screens::draw_sound_settings` (0x47022A): the sound page's items 4 .. 6 (3D sound, EAX,
//! A3D 2) choosable only when the card has them (`g_audioOptions.has_3d`, `has_eax`,
//! `has_a3d2`), and the six settings' names (strings 0x54B + quality, 0x554 + channels,
//! 0x56D + DS3D, EAX, A3D, Dolby; each kept in `dword_7559EC`) at x 0x1D1, y 0x10B and on
//! every 0x1B, in the font at 0x51E8C4.
//!
//! In:  nothing
//! Out: as the last `draw_string_by_id` leaves them (carry, eax 0, ebx, ecx, edx)
//!
//! In a menu loop's draw list (0x758C96). Refuses what `menu::enable_item` refuses (no current
//! menu), before anything. The menus and draw lists run its `entry` (`menu::table_calls`); the
//! registers above are the original's, for reference.

use crate::audio::store::AUDIO;
use crate::errors::Refused;
use crate::framework::{game_read, rva};
use crate::menu::disable_item::disable_item;
use crate::menu::enable_item::enable_item;
use crate::text::draw_text::Canvas;

fn k(a: u32) -> u32 {
    unsafe { game_read(rva(a as usize)) }
}

/// (the setting, its first string).
const NAMES: [(u32, u32); 6] = [
    (0x75566C, 0x54B),
    (0x755680, 0x554),
    (0x755670, 0x56D),
    (0x755674, 0x56D),
    (0x755678, 0x56D),
    (0x75567C, 0x56D),
];

/// The Rust version (the registers the last name leaves: the menus read none of them).
pub unsafe fn draw_sound_settings(canvas: &mut dyn Canvas) -> Result<(), Refused> {
    // Converted: `rc2k_core::screens::settings_pages::{sound_items, sound}`.
    use crate::screens::game_screen::SettingsScreen;
    // The switches, by their items 4, 5 and 6: 3D, EAX and A3D2 there.
    let options = AUDIO.audio_options.get();
    let [a, b, c] = [options.has_3d, options.has_eax, options.has_a3d2].map(|switch| switch == 1);
    for (item, on) in rc2k_core::screens::settings_pages::sound_items(a, b, c) {
        if on {
            enable_item(item)?;
        } else {
            disable_item(item)?;
        }
    }
    let ids = NAMES.map(|(setting, first)| k(setting).wrapping_add(first));
    rc2k_core::screens::settings_pages::sound(&mut SettingsScreen::new(canvas), ids)?;
    Ok(())
}
