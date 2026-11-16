//! `menu::disable_item`: an item of the current menu greyed out.
//!
//! The item (`menu::item_at`): +0x48 = 0xFFFFFC1C (the moves up and down pass over it) and its font
//! +0x18 = the grey one. Its twin `menu::enable_item` puts 0xFFFFFC1B and the font back. Refuses
//! item 0.

use crate::errors::Refused;
use crate::menu::item_at::item_at;
use crate::menu::owner::Menus;

use rc2k_core::menu::items::{Font, choosable};

use crate::gfx::typefaces::fonts::{ITEM_GREY as GREY_FONT, ITEM_PLAIN as PLAIN_FONT};

/// The item's +0x48 and font set: what both twins do.
pub fn set_item(menus: &Menus, number: u32, on: bool) -> Result<(), Refused> {
    let item = item_at(menus, number)?;
    let (state, font) = choosable(on);
    let font = match font {
        Font::Plain => PLAIN_FONT,
        Font::Grey => GREY_FONT,
    };
    item.skip_mark.set(state);
    item.font.set(font);
    Ok(())
}

pub fn disable_item(menus: &Menus, number: u32) -> Result<(), Refused> {
    set_item(menus, number, false)
}
