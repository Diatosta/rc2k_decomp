//! `menu::load_slideshow_picture` (0x4684F5): the slideshow's picture of the car shown,
//! the next one that is there.
//!
//! In:  eax = the way to look (below 0: back, else on)
//! Out: every register preserved
//!
//! The car (as `menu::load_car_background` finds it, 0x21 counting as 0x17); its pictures are
//! `off_759B0E[car]` (`;3files\g\slides\mgnslNN.png` and so on) with the number `dword_755994`.
//! Starting from that number: the path made (`files::numbered_path`); if the file is there it is
//! taken, else the path's last two characters become "cx" (`.pcx`) and if that is there it is
//! taken; else the number steps the way (from 0 back to 99, from 99 on to 0) and the next is tried,
//! until the number comes round to where it started, when the last path tried is taken anyway. Then
//! `gfx::load_picture_with_overlay` with it and the title bar
//! (`files\g\graphics\screens\titlebar.png`), or, if that could not read it, `gfx::load_picture` of
//! the title bar alone; then `gfx::convert_picture`. Five call sites: the slideshow's buttons and
//! its set-up. Refuses a car with no pictures (only the first 0x18 have them), and what its parts
//! refuse.

use crate::errors::Refused;
use crate::files::file_exists::file_exists;
use crate::files::numbered_path::numbered_path_in_game;
use crate::gfx::convert_picture::convert_picture_now;
use crate::gfx::load_picture::load_picture;
use crate::gfx::load_picture_with_overlay::load_picture_with_overlay;
use crate::menu::event_store::EVENT_STATE;
use crate::menu::lists_store::MENU_LISTS;
use crate::menu::load_car_background::shown_car;
use crate::strings::text_length::text_length;

// ---- game data ----

/// What it asks for.
pub enum Step {
    /// `gfx::load_picture_with_overlay` with the path and the title bar: the carry.
    Overlay(*mut u8, *mut u8),
    /// `gfx::load_picture` of the title bar.
    Load(*mut u8),
    Convert,
}

// ---- Rust version ----

pub unsafe fn load_slideshow_picture(
    way: u32,
    step: &mut dyn FnMut(Step) -> Result<bool, Refused>,
) -> Result<(), Refused> {
    unsafe {
        // Converted: `rc2k_core::menu::slides` (`slideshow_car`, `search`, `pcx`).
        use rc2k_core::menu::slides;
        let car = slides::slideshow_car(shown_car()?)?;
        let template = MENU_LISTS.slideshow_templates[car as usize].get();
        let start: u32 = EVENT_STATE.picture.get();
        let mut path = std::ptr::null_mut();
        slides::search(
            start,
            (way as i32) < 0,
            |number| -> Result<bool, Refused> {
                path = numbered_path_in_game(template as *const u8, number)? as *mut u8;
                if file_exists(path)? {
                    return Ok(true);
                }
                let length = text_length(path)? as usize;
                slides::pcx(std::slice::from_raw_parts_mut(path, length))?;
                Ok(file_exists(path)?)
            },
            |next| EVENT_STATE.picture.set(next),
        )?;
        let title_bar = crate::menu::texts_store::MENU_TEXTS.titlebar_png.addr() as *mut u8;
        if step(Step::Overlay(path, title_bar))? {
            step(Step::Load(title_bar))?;
        }
        step(Step::Convert)?;
        Ok(())
    }
}

/// With the Rust versions of what it calls.
pub unsafe fn load_slideshow_picture_in_game(way: u32) -> Result<(), Refused> {
    unsafe {
        load_slideshow_picture(way, &mut |step| match step {
            Step::Overlay(path, over) => load_picture_with_overlay(path, over),
            Step::Load(path) => load_picture(path).map(|_| false),
            Step::Convert => convert_picture_now().map(|_| false).map_err(Into::into),
        })
    }
}
