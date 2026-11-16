//! Screens no route reaches, asked for from the debug UI ("Screens" tab) and run from the
//! menu loop's next frame (`menu::screen_loop`): the game's flow stays what it was, a
//! screen is entered as from a menu item.

use crate::errors::Refused;
use crate::menu::screen_id::ScreenId;
use crate::owners::Owners;
#[cfg(debug_ui)]
use crate::saves::owner::Saves;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

/// A race has been played this session (`drive::play_race`).
static RACED: AtomicBool = AtomicBool::new(false);
/// The Developers tab's free play should start at the start (`race::car_starts`).
static PLACE_FREE_PLAY: AtomicBool = AtomicBool::new(false);

/// A race has been played.
pub fn note_race() {
    RACED.store(true, Ordering::Relaxed);
}

/// Whether a race has been played.
#[cfg(debug_ui)]
pub fn raced() -> bool {
    RACED.load(Ordering::Relaxed)
}

/// Says whether free play starts at the start.
#[cfg(debug_ui)]
pub fn place_free_play(on: bool) {
    PLACE_FREE_PLAY.store(on, Ordering::Relaxed);
}

/// Whether free play starts at the start.
pub fn free_play_placed() -> bool {
    PLACE_FREE_PLAY.load(Ordering::Relaxed)
}

/// The car specification screen's number in `WANTED`.
const CSPECS: u32 = 100;

/// What is asked for: 0 nothing, `CSPECS`, else a section's best times + 1.
static WANTED: AtomicU32 = AtomicU32::new(0);

/// The section best times of `section` (0 .. 5) at the menu's next frame.
#[cfg(debug_ui)]
pub fn ask_section_times(section: u32) {
    WANTED.store(section + 1, Ordering::Relaxed);
}

/// The car specification screen at the menu's next frame.
#[cfg(debug_ui)]
pub fn ask_cspecs() {
    WANTED.store(CSPECS, Ordering::Relaxed);
}

/// Whether the section has its table (the draw refuses one without).
#[cfg(debug_ui)]
pub fn section_has_times(saves: &Saves, section: usize) -> bool {
    saves.rally_times.table_named(section).is_ok()
}

/// A menu's frame: runs what was asked for; true if a screen was run (the menu sets
/// itself up again, its background having been replaced).
pub fn serve(o: Owners<'static>) -> Result<bool, Refused> {
    let screens = &o.menus.screens;
    #[cfg(debug_ui)]
    if crate::debug::dev_front_end::serve(o)? {
        return Ok(true);
    }
    let wanted = WANTED.swap(0, Ordering::Relaxed);
    if wanted == 0 {
        return Ok(false);
    }
    if wanted == CSPECS {
        crate::menu::screen_loop::screen_entry(o, ScreenId::Cspecs)?;
        return Ok(true);
    }
    let section = screens.event.section.get();
    screens.event.section.set(wanted - 1);
    let result = crate::menu::screen_loop::screen_entry(o, ScreenId::SectionTimes);
    screens.event.section.set(section);
    result?;
    Ok(true)
}
