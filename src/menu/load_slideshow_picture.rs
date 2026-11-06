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
use crate::files::file_exists::file_exists_at;
use crate::files::numbered_path::numbered_path_in_game;
use crate::gfx::convert_picture::convert_picture_now;
use crate::gfx::load_picture::load_picture;
use crate::gfx::load_picture_with_overlay::load_picture_with_overlay;
use crate::menu::load_car_background::shown_car;
use crate::menu::owner::menus;
use crate::pointers::GamePtr;

// ---- game data ----

/// What it asks for.
pub enum Step {
    /// `gfx::load_picture_with_overlay` with the path and the title bar: the carry.
    Overlay(GamePtr, GamePtr),
    /// `gfx::load_picture` of the title bar.
    Load(GamePtr),
    Convert,
}

// ---- Rust version ----

pub fn load_slideshow_picture(
    way: u32,
    step: &mut dyn FnMut(Step) -> Result<bool, Refused>,
) -> Result<(), Refused> {
    let menus = menus();
    // Converted: `rc2k_core::menu::slides` (`slideshow_car`, `search`, `pcx`).
    use rc2k_core::menu::slides;
    let car = slides::slideshow_car(shown_car()?)?;
    let template = crate::menu::texts_store::text_at(
        menus.tables.lists.slideshow_templates[car as usize].get(),
    )?;
    let start: u32 = menus.screens.event.picture.get();
    // Set by the first try (`search` always makes one).
    let mut path = None;
    slides::search(
        start,
        (way as i32) < 0,
        |number| -> Result<bool, Refused> {
            let at = numbered_path_in_game(template, number)?;
            path = Some(at);
            if file_exists_at(at)? {
                return Ok(true);
            }
            let length = crate::strings::text_length::nul_at(at, "a picture path")? as usize;
            let mut text = at.read_bytes(length, "a picture path")?;
            slides::pcx(&mut text)?;
            at.write_bytes(&text, "a picture path")?;
            Ok(file_exists_at(at)?)
        },
        |next| menus.screens.event.picture.set(next),
    )?;
    let path = path.ok_or(crate::errors::BadInput("no slideshow picture tried"))?;
    let title_bar = GamePtr::to(&menus.tables.texts.titlebar_png);
    if step(Step::Overlay(path, title_bar))? {
        step(Step::Load(title_bar))?;
    }
    step(Step::Convert)?;
    Ok(())
}

/// With the Rust versions of what it calls.
pub fn load_slideshow_picture_in_game(way: u32) -> Result<(), Refused> {
    load_slideshow_picture(way, &mut |step| match step {
        Step::Overlay(path, over) => load_picture_with_overlay(&path, &over),
        Step::Load(path) => load_picture(&path).map(|_| false),
        Step::Convert => convert_picture_now().map(|_| false).map_err(Into::into),
    })
}
