//! `menu::load_car_background` (0x468482): the car specification page's background,
//! the car's own.
//!
//! In:  nothing
//! Out: every register preserved; carry as `menu::load_background` leaves it
//!
//! The car is the first dword of the shown entry's record (`dword_755990` through
//! `dword_75A65C`), 0x21 counting as 0x17; the path is that car's in the page's table
//! (`off_759AD2[dword_755998]`); `menu::load_background` with it and ebx = the car (the
//! blit's palette). A carry after it is fatal error 0x80712345. Five call sites, the
//! car specification screens' set-ups.
//! Shadow mode runs the original alone (it loads a picture).
//! Refuses a page past the four, and what the record's lookup refuses.
//! Self-test: a redirected copy whose `load_background` notes what it gets, every page
//! with the game's records.

use crate::errors::{FatalError, Refused};
use crate::framework::{game_read, rva};
use crate::menu::car_description::{read, shown_description};
use crate::menu::load_background::load_background;
use crate::text::draw_text::GameCanvas;

// ---- game data ----

const PAGE_RVA: usize = rva(0x755998); // dword_755998
// dword_755990
const PATHS_RVA: usize = rva(0x759AD2); // off_759AD2: a table of paths a page
// menu::load_background

// ---- Rust version ----

/// The car of the entry shown, 0x21 counting as 0x17.
pub fn shown_car() -> Result<u32, Refused> {
    let car = unsafe { read(shown_description()?) }.car;
    // Converted: `rc2k_core::menu::slides::picture_car`.
    Ok(rc2k_core::menu::slides::picture_car(car))
}

/// `load` gets the path and the car; it returns whether the carry was set.
pub unsafe fn load_car_background(
    load: &mut dyn FnMut(u32, u32) -> Result<bool, Refused>,
) -> Result<(), Refused> {
    unsafe {
        // Converted: `rc2k_core::menu::slides::background_page`.
        let page = rc2k_core::menu::slides::background_page(game_read(PAGE_RVA))?;
        let car = shown_car()?;
        let table: u32 = game_read(PATHS_RVA + page as usize * 4);
        let path = ((table + car * 4) as *const u32).read_unaligned();
        if load(path, car)? {
            return Err(FatalError {
                code: 0x8071_2345,
                message: "",
            }
            .into());
        }
        Ok(())
    }
}

/// In the game: the background through `menu::load_background` (its carry is never set).
pub unsafe fn load_car_background_in_game() -> Result<(), Refused> {
    unsafe {
        load_car_background(&mut |path, car| {
            load_background(path as *mut u8, car, &mut GameCanvas)?;
            Ok(false)
        })
    }
}
