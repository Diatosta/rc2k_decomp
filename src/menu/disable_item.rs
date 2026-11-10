//! `menu::disable_item` (0x46B4F6): an item of the current menu greyed out.
//!
//! The item (`menu::item_at`): +0x48 = 0xFFFFFC1C (the moves up and down pass over it) and its font
//! +0x18 = the grey one at 0x51EA04. Its twin `menu::enable_item` (0x46B50F) puts 0xFFFFFC1B and
//! the font at 0x51E8E4 back. Eight callers each (the player details screens: the controller's item
//! when there is no controller). Refuses item 0.

use crate::errors::Refused;
use crate::menu::item_at::item_at;
use crate::menu::owner::Screens;
use crate::menu::owner::menus;

use rc2k_core::menu::items::{Font, choosable};

use crate::gfx::typefaces::fonts::{ITEM_GREY as GREY_FONT, ITEM_PLAIN as PLAIN_FONT};

// Converted: `rc2k_core::menu::items::choosable`.

/// The item's +0x48 and font set: what both twins do.
pub fn set_item(screens: &Screens, number: u32, on: bool) -> Result<(), Refused> {
    let item = item_at(screens, number)?;
    let (state, font) = choosable(on);
    let font = match font {
        Font::Plain => PLAIN_FONT,
        Font::Grey => GREY_FONT,
    };
    item.skip_mark.set(state);
    item.font.set(font);
    Ok(())
}

pub fn disable_item(number: u32) -> Result<(), Refused> {
    let screens = &menus().screens;
    set_item(screens, number, false)
}
