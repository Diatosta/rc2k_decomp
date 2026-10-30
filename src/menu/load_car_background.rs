//! `menu::load_car_background` (0x468482): the car specification page's background,
//! the car's own.
//!
//! In:  nothing
//! Out: every register preserved; carry as `menu::load_background` leaves it
//!
//! The car is the first dword of the shown entry's record (`dword_755990` through `dword_75A65C`),
//! 0x21 counting as 0x17; the path is that car's in the page's table (`off_759AD2[dword_755998]`);
//! `menu::load_background` with it and ebx = the car (the blit's palette). A carry after it is
//! fatal error 0x80712345. Five call sites, the car specification screens' set-ups. Refuses a page
//! past the four, and what the record's lookup refuses.

use crate::errors::{FatalError, Refused};
use crate::menu::car_description::{read, shown_description};
use crate::menu::event_store::EVENT_STATE;
use crate::menu::load_background::load_background;
use crate::text::draw_text::GameCanvas;

// ---- game data ----

// dword_755990
// menu::load_background

// ---- Rust version ----

/// The car of the entry shown, 0x21 counting as 0x17.
pub fn shown_car() -> Result<u32, Refused> {
    let car = read(shown_description()?)?.car;
    // Converted: `rc2k_core::menu::slides::picture_car`.
    Ok(rc2k_core::menu::slides::picture_car(car))
}

/// `load` gets the path and the car; it returns whether the carry was set.
pub fn load_car_background(
    load: &mut dyn FnMut(u32, u32) -> Result<bool, Refused>,
) -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::slides::background_page`.
    let page = rc2k_core::menu::slides::background_page(EVENT_STATE.car_page.get())?;
    let car = shown_car()?;
    let path = crate::menu::texts_store::text_at(crate::session::read_on(
        crate::menu::lists_store::page_pictures(page)?,
        car as usize,
        "load car background: a game value",
    )?)?
    .addr();
    if load(path, car)? {
        return Err(FatalError {
            code: 0x8071_2345,
            message: "",
        }
        .into());
    }
    Ok(())
}

/// In the game: the background through `menu::load_background` (its carry is never set).
pub fn load_car_background_in_game() -> Result<(), Refused> {
    load_car_background(&mut |path, car| {
        load_background(&crate::pointers::GamePtr::new(path), car, &mut GameCanvas)?;
        Ok(false)
    })
}
