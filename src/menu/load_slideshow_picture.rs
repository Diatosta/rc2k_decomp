//! `menu::load_slideshow_picture` (0x4684F5): the slideshow's picture of the car shown,
//! the next one that is there.
//!
//! The car (as `menu::load_car_background` finds it, 0x21 counting as 0x17); its pictures are
//! the slideshow picture paths (`;3files\g\slides\mgnslNN.png` and so on) with the number `event.picture`.
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
use crate::files::numbered_path::numbered_path_bytes;
use crate::gfx::convert_picture::convert_picture_now_in;
use crate::gfx::load_picture::load_picture_with;
use crate::gfx::load_picture_with_overlay::load_picture_with_overlay_in;
use crate::menu::load_car_background::shown_car;
use crate::menu::owner::Menus;
use crate::owners::Owners;

// ---- game data ----

/// What it asks for.
pub enum Step<'a> {
    /// `gfx::load_picture_with_overlay` with the path (its text, without the NUL) and the
    /// title bar: the carry.
    Overlay(Vec<u8>, &'a dyn crate::session::TextPlace),
    /// `gfx::load_picture` of the title bar.
    Load(&'a dyn crate::session::TextPlace),
    Convert,
}

pub fn load_slideshow_picture(
    menus: &Menus,
    way: u32,
    step: &mut dyn FnMut(Step<'_>) -> Result<bool, Refused>,
) -> Result<(), Refused> {
    // The logic is in `rc2k_core::menu::slides` (`slideshow_car`, `search`, `pcx`).
    use rc2k_core::menu::slides;
    let car = slides::slideshow_car(shown_car(menus)?)?;
    let template = crate::menu::texts_store::text_at(
        menus.tables.lists.slideshow_templates[car as usize].get(),
    )?
    .text("a path template")?;
    let start: u32 = menus.screens.event.picture.get();
    // Set by the first try (`search` always makes one).
    let mut path: Option<Vec<u8>> = None;
    slides::search(
        start,
        (way as i32) < 0,
        |number| -> Result<bool, Refused> {
            // The path is also written to `PATHS`, as the original builds it there.
            let numbered = numbered_path_bytes(&template, number)?;
            let there = file_exists(&numbered)?;
            path = Some(numbered.clone());
            if there {
                return Ok(true);
            }
            let mut text = numbered;
            slides::pcx(&mut text)?;
            crate::files::state::PATHS.write_at(0, &text, "a picture path")?;
            let exists = file_exists(&text)?;
            path = Some(text);
            Ok(exists)
        },
        |next| menus.screens.event.picture.set(next),
    )?;
    let path = path.ok_or(crate::errors::BadInput("no slideshow picture tried"))?;
    let title_bar = &menus.tables.texts.titlebar_png;
    if step(Step::Overlay(path, title_bar))? {
        step(Step::Load(title_bar))?;
    }
    step(Step::Convert)?;
    Ok(())
}

/// With the game's own steps for what it calls.
pub fn load_slideshow_picture_in_game(_menus: &Menus, way: u32) -> Result<(), Refused> {
    load_slideshow_picture_in(crate::owners::boundary(), way)
}

/// `load_slideshow_picture_in_game` for a caller that holds the owners.
pub fn load_slideshow_picture_in(o: Owners<'_>, way: u32) -> Result<(), Refused> {
    load_slideshow_picture(o.menus, way, &mut |step| match step {
        Step::Overlay(path, over) => {
            load_picture_with_overlay_in(&MirroredPath::new(path), over, o.gfx, o.system, o.strings)
        }
        Step::Load(path) => load_picture_with(path, o.gfx, o.system).map(|_| false),
        Step::Convert => convert_picture_now_in(o.gfx, o.system)
            .map(|_| false)
            .map_err(Into::into),
    })
}

/// A picture's path as a value, for the loader that may swap its ending: the swap is
/// written to the value and, as the original swapped it in place, to `PATHS` too.
struct MirroredPath(std::cell::RefCell<Vec<u8>>);

impl MirroredPath {
    fn new(path: Vec<u8>) -> Self {
        Self(std::cell::RefCell::new(path))
    }
}

impl crate::session::TextPlace for MirroredPath {
    fn text(&self, _what: &str) -> Result<Vec<u8>, crate::errors::BadInput> {
        Ok(self.0.borrow().clone())
    }

    fn write_text_at(
        &self,
        at: usize,
        bytes: &[u8],
        what: &str,
    ) -> Result<(), crate::errors::BadInput> {
        let mut value = self.0.borrow_mut();
        let end = at + bytes.len();
        if end > value.len() {
            return Err(crate::errors::BadInput("a write past a path's end"));
        }
        value[at..end].copy_from_slice(bytes);
        crate::files::state::PATHS.write_at(at, bytes, what)
    }
}
