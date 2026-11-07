//! `screens::draw_graphics_settings` (0x46FDF0): the graphics page: item 2 choosable only
//! when `dword_4F50D4` + `dword_4F50D8` is 2; the settings' names (strings by the work
//! values, each kept in `dword_7559EC`) and the screen mode "w x h x bits" (`unk_4F50E8`
//! [`dword_4F50C8`], built in the shared text buffer) at x 0x1D1 from y 0xDC every 0x1B, in
//! the font at 0x51E8C4.
//!
//!
//! In a menu loop's draw list (0x758CB2). Refuses what `menu::enable_item` refuses (no current
//! menu), before anything. The menus and draw lists run its `entry` (`menu::table_calls`).

use crate::errors::Refused;
use crate::gfx::owner::gfx;
use crate::menu::disable_item::disable_item;
use crate::menu::enable_item::enable_item;
use crate::menu::owner::menus;
use crate::text::draw_text::Canvas;

/// (the setting, its first string, its row) for the names; row 1 is the screen mode.
type Name = (fn() -> u32, u32, i32);

const NAMES: [Name; 8] = [
    (|| gfx().display.choices.card.get(), 0x38F, 0),
    (|| gfx().display.choices.buffering.get(), 0x391, 2),
    (|| gfx().display.choices.mipmapping.get(), 0x56D, 3),
    (|| menus().screens.option_work.setting_0.get(), 0x56D, 4),
    (|| menus().screens.option_work.setting_1.get(), 0x56D, 5),
    (|| menus().screens.option_work.setting_2.get(), 0x396, 6),
    (|| menus().screens.option_work.setting_3.get(), 0x393, 7),
    (|| menus().screens.option_work.setting_4.get(), 0x394, 8),
];

/// The Rust version (the registers the last name leaves: the menus read none of them).
pub fn draw_graphics_settings(canvas: &mut dyn Canvas) -> Result<(), Refused> {
    let display = &gfx().display;
    // Converted: `rc2k_core::screens::settings_pages::{graphics_item, graphics}`.
    use crate::screens::game_screen::SettingsScreen;
    {
        if rc2k_core::screens::settings_pages::graphics_item(
            display.choices.has_primary.get(),
            display.choices.has_secondary.get(),
        ) {
            enable_item(2)?;
        } else {
            disable_item(2)?;
        }
        let place = display.choices.resolution.get();
        let v = |i: usize| display.choices.list.dword(place, i, "a screen mode");
        let ids = NAMES.map(|(setting, first, _)| setting().wrapping_add(first));
        rc2k_core::screens::settings_pages::graphics(
            &mut SettingsScreen::new(canvas),
            ids,
            [v(0)?, v(1)?, v(2)?],
        )?;
        Ok(())
    }
}
