//! `menu::run_front_end` (0x47486C): the game between its start and its end: the films
//! and the main menu, with everything that is reached from it.
//!
//! `audio.levels.master_volume_shown` = the master volume; `menu::drop_missing_controllers`; `gfx::build_fonts`;
//! the films at the start (`menu::play_films` with eax = 0, not ported); the log line "BEV:
//! Starting GB_Front"; `menu::main_menu_loop`; "BEV: Finished GB_Front";
//! `menu::store_player_names`; `gfx::free_fonts`; `saves::list_car_numbers`.
//! Callers: `system::main_loop` and 0x443260. Refuses what its parts refuse; such a refusal comes
//! with the work half done. Rust callers run its `entry` (`routes.rs`).

use crate::errors::Refused;
use crate::framework::log;
use crate::menu::screen_loop::{Entry, run_screen};
use crate::owners::Owners;

// ---- game data ----

const STARTING_RVA: &std::ffi::CStr = c"BEV: Starting GB_Front"; // "BEV: Starting GB_Front"
const FINISHED_RVA: &std::ffi::CStr = c"BEV: Finished GB_Front"; // "BEV: Finished GB_Front"

pub fn run_front_end(o: Owners<'_>) -> Result<(), Refused> {
    let menus = o.menus;
    let audio = o.audio;
    let gfx = o.gfx;
    let fonts = &gfx.fonts;
    let play = &menus.play;
    log("menu::run_front_end: runs");
    audio
        .levels
        .master_volume_shown
        .set(audio.state.master_volume.get());
    crate::menu::drop_missing_controllers::drop_missing_controllers(menus, o.input)?;
    crate::gfx::build_fonts::build_fonts_in(gfx, o.system, o.strings)?;
    // The films, with eax 0 (the game's start).
    crate::menu::play_films::play_films(o, false)?;
    crate::logfile::log_write::log_nul(STARTING_RVA, true, 1);
    run_screen(&crate::menu::main_menu_loop::SCREEN, &Entry::new(o))?;
    crate::logfile::log_write::log_nul(FINISHED_RVA, true, 1);
    crate::menu::store_player_names::store_player_names(menus, play, &mut |line| {
        crate::logfile::log_write::log_nul(line, true, 1);
    })?;
    crate::gfx::free_fonts::free_fonts(&fonts.glyph_tables.memory, &fonts.font_block.palettes)?;
    crate::saves::list_car_numbers::list_car_numbers_in(menus)?;
    log("menu::run_front_end: done");
    Ok(())
}
