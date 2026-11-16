//! `menu::load_car_background`: the car specification page's background, the car's own.
//!
//! The car is the first dword of the shown entry's record (`event.shown.car` through the
//! carousel's car list), 0x21 counting as 0x17; the path is that car's in the page's table (the
//! four page lists); `menu::load_background` shows it. Refuses a page past the four, and what the
//! record's lookup refuses.

use crate::errors::Refused;
use crate::menu::car_description::{read, shown_description};
use crate::menu::load_background::load_background_in;
use crate::menu::owner::Menus;
use crate::text::draw_text::GameCanvas;

/// The car of the entry shown, 0x21 counting as 0x17.
pub fn shown_car(menus: &Menus) -> Result<u32, Refused> {
    let car = read(menus, shown_description(menus)?)?.car;
    Ok(rc2k_core::menu::slides::picture_car(car))
}

pub fn load_car_background(
    menus: &'static Menus,
    gfx: &'static crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    let page = rc2k_core::menu::slides::background_page(menus.screens.event.shown.car_page.get())?;
    let car = shown_car(menus)?;
    let path = crate::menu::texts_store::text_at(
        &menus.texts,
        menus.lists.page_picture(page, car as usize)?,
    )?;
    load_background_in(&path, &mut GameCanvas::new(gfx), menus, gfx, system)
}
