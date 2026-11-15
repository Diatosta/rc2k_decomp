//! `menu::load_car_background`: the car specification page's background, the car's own.
//!
//! The car is the first dword of the shown entry's record (`event.shown.car` through the
//! carousel's car list), 0x21 counting as 0x17; the path is that car's in the page's table (the
//! four page lists); `menu::load_background` runs with it and the car (the blit's palette). A
//! failed load is fatal error 0x80712345. Five call sites, the car specification
//! screens' set-ups. Refuses a page past the four, and what the record's lookup refuses.

use crate::errors::{FatalError, Refused};
use crate::menu::car_description::{read, shown_description};
use crate::menu::lists_store::lists;
use crate::menu::load_background::load_background_in;
use crate::menu::owner::Menus;
use crate::text::draw_text::GameCanvas;

// `event.shown.car`
// menu::load_background

/// The car of the entry shown, 0x21 counting as 0x17.
pub fn shown_car(menus: &Menus) -> Result<u32, Refused> {
    let car = read(menus, shown_description(menus)?)?.car;
    Ok(rc2k_core::menu::slides::picture_car(car))
}

/// `load` gets the path; it returns whether the load failed. (The car is also handed to it
/// as the frame sprite's palette, which the sprite never looks at.)
pub fn load_car_background(
    menus: &'static Menus,
    load: &mut dyn FnMut(crate::place::BytePlace) -> Result<bool, Refused>,
) -> Result<(), Refused> {
    let page = rc2k_core::menu::slides::background_page(menus.screens.event.shown.car_page.get())?;
    let car = shown_car(menus)?;
    let path = crate::menu::texts_store::text_at(lists().page_picture(page, car as usize)?)?;
    if load(path)? {
        return Err(FatalError {
            code: 0x8071_2345,
            message: "",
        }
        .into());
    }
    Ok(())
}

/// In the game: the background through `menu::load_background` (which never fails).
pub fn load_car_background_in_game(
    menus: &'static Menus,
    gfx: &'static crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    load_car_background(menus, &mut |path| {
        load_background_in(&path, &mut GameCanvas::new(gfx), menus, gfx, system)?;
        Ok(false)
    })
}
