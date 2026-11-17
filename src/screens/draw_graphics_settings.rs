//! The graphics page: item 2 is choosable only when the card has both a primary and a
//! secondary display; each setting's name is a string picked by its value, with the screen mode
//! ("w x h x bits") on row 1. Refuses what `menu::enable_item` refuses (no current menu).

use crate::errors::Refused;
use crate::gfx::owner::Gfx;
use crate::menu::disable_item::disable_item;
use crate::menu::enable_item::enable_item;
use crate::menu::owner::Menus;
use crate::text::draw_text::Canvas;

/// (the setting, its first string, its row) for the names; row 1 is the screen mode.
type Name = (fn(&Menus, &Gfx) -> u32, u32, i32);

const NAMES: [Name; 8] = [
    (|_, gfx| gfx.display.choices.card.get(), 0x38F, 0),
    (|_, gfx| gfx.display.choices.buffering.get(), 0x391, 2),
    (|_, gfx| gfx.display.choices.mipmapping.get(), 0x56D, 3),
    (|menus, _| menus.options.lens_flare.get(), 0x56D, 4),
    (|menus, _| menus.options.reflections.get(), 0x56D, 5),
    (|menus, _| menus.options.car_shadows.get(), 0x396, 6),
    (|menus, _| menus.options.car_detail.get(), 0x393, 7),
    (|menus, _| menus.options.headlights.get(), 0x394, 8),
];

/// The frame limit as the page shows it: "off", "display (72 fps)" or "30 fps".
fn frame_limit_text(o: crate::owners::Owners<'static>) -> Vec<u8> {
    use crate::strings::append_string_by_id::own_text;
    use rc2k_core::gfx::frame_limit::{FrameLimit, display_rate};
    use rc2k_core::strings::format::{Pad, decimal};
    use rc2k_core::strings::own::id;
    let own = |n: u32| own_text(o.strings, n);
    let rate = |frames: u32| {
        let mut text = decimal(frames, Pad::None);
        text.push(b' ');
        text.extend(own(id::FRAMES_A_SECOND));
        text
    };
    match crate::platform::pacing::frame_limit() {
        FrameLimit::Off => own(id::NO_LIMIT),
        FrameLimit::Rate(n) => rate(n),
        FrameLimit::Display => {
            let shown = display_rate(crate::platform::pacing::display_refresh());
            let mut text = own(id::DISPLAY);
            text.extend(b" (");
            text.extend(rate(shown.round() as u32));
            text.push(b')');
            text
        }
    }
}

pub fn draw_graphics_settings(
    o: crate::owners::Owners<'static>,
    gfx: &crate::gfx::owner::Gfx,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    let menus = o.menus;
    let display = &gfx.display;
    // The logic is in `rc2k_core::screens::settings_pages::{graphics_item, graphics}`.
    use crate::screens::game_screen::SettingsScreen;
    {
        if rc2k_core::screens::settings_pages::graphics_item(
            display.choices.has_primary.get(),
            display.choices.has_secondary.get(),
        ) {
            enable_item(menus, 2)?;
        } else {
            disable_item(menus, 2)?;
        }
        let place = display.choices.resolution.get();
        let v = |i: usize| display.choices.list.dword(place, i, "a screen mode");
        let ids = NAMES.map(|(setting, first, _)| setting(menus, gfx).wrapping_add(first));
        let mut screen = SettingsScreen::new(canvas, o);
        rc2k_core::screens::settings_pages::graphics(&mut screen, ids, [v(0)?, v(1)?, v(2)?])?;
        rc2k_core::screens::settings_pages::graphics_frame_limit(
            &mut screen,
            &frame_limit_text(o),
        )?;
        Ok(())
    }
}
