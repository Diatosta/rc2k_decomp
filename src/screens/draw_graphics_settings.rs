//! The graphics page: item 2 is choosable only when the card has both a primary and a
//! secondary display; each setting's name is a string picked by its value, drawn at x 0x1D1
//! from y 0xDC every 0x1B in the item font, with the screen mode ("w x h x bits", built in
//! the shared text buffer) on row 1.
//!
//! In a menu loop's draw list. Refuses what `menu::enable_item` refuses (no current menu),
//! before anything.

use crate::errors::Refused;
use crate::gfx::owner::gfx;
use crate::menu::disable_item::disable_item;
use crate::menu::enable_item::enable_item;
use crate::menu::owner::Menus;
use crate::text::draw_text::Canvas;

/// (the setting, its first string, its row) for the names; row 1 is the screen mode.
type Name = (fn(&Menus) -> u32, u32, i32);

const NAMES: [Name; 8] = [
    (|_| gfx().display.choices.card.get(), 0x38F, 0),
    (|_| gfx().display.choices.buffering.get(), 0x391, 2),
    (|_| gfx().display.choices.mipmapping.get(), 0x56D, 3),
    (|menus| menus.screens.option_work.lens_flare.get(), 0x56D, 4),
    (
        |menus| menus.screens.option_work.reflections.get(),
        0x56D,
        5,
    ),
    (
        |menus| menus.screens.option_work.car_shadows.get(),
        0x396,
        6,
    ),
    (|menus| menus.screens.option_work.car_detail.get(), 0x393, 7),
    (|menus| menus.screens.option_work.headlights.get(), 0x394, 8),
];

pub fn draw_graphics_settings(menus: &Menus, canvas: &mut dyn Canvas) -> Result<(), Refused> {
    let display = &gfx().display;
    // Converted: `rc2k_core::screens::settings_pages::{graphics_item, graphics}`.
    use crate::screens::game_screen::SettingsScreen;
    {
        if rc2k_core::screens::settings_pages::graphics_item(
            display.choices.has_primary.get(),
            display.choices.has_secondary.get(),
        ) {
            enable_item(&menus.screens, 2)?;
        } else {
            disable_item(&menus.screens, 2)?;
        }
        let place = display.choices.resolution.get();
        let v = |i: usize| display.choices.list.dword(place, i, "a screen mode");
        let ids = NAMES.map(|(setting, first, _)| setting(menus).wrapping_add(first));
        rc2k_core::screens::settings_pages::graphics(
            &mut SettingsScreen::new(canvas),
            ids,
            [v(0)?, v(1)?, v(2)?],
        )?;
        Ok(())
    }
}
