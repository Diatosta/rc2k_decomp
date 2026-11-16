//! `menu::run_front_end`: the game between its start and its end: the films and the main menu,
//! with everything that is reached from it.
//!
//! `audio.levels.general_volume` = the master volume; `menu::drop_missing_controllers`;
//! `gfx::build_fonts`; the films at the start (`menu::play_films`); the log line "BEV: Starting
//! GB_Front"; `menu::main_menu_loop`; "BEV: Finished GB_Front"; `menu::store_player_names`;
//! `gfx::free_fonts`; `saves::list_car_numbers`. A race asked for on the command line
//! (`menu::launch`) skips the films; the main menu's set-up runs it. Refuses what its parts refuse;
//! such a refusal comes with the work half done.

use crate::errors::Refused;
use crate::logfile::program_log::log;
use crate::menu::screen_id::ScreenId;
use crate::menu::screen_loop::{Entry, run_screen};
use crate::owners::Owners;

const STARTING: &std::ffi::CStr = c"BEV: Starting GB_Front";
const FINISHED: &std::ffi::CStr = c"BEV: Finished GB_Front";

pub fn run_front_end(o: Owners<'static>) -> Result<(), Refused> {
    let menus = o.menus;
    let audio = o.audio;
    let gfx = o.gfx;
    let fonts = &gfx.fonts;
    let play = &menus.play;
    log("menu::run_front_end: runs");
    audio
        .levels
        .general_volume
        .set(audio.state.master_volume.get());
    crate::menu::drop_missing_controllers::drop_missing_controllers(menus, o.input)?;
    crate::gfx::build_fonts::build_fonts_in(gfx, o.system, o.strings)?;
    // The films, at the game's start, unless a race or a replay was asked for on the command line (the
    // main menu's set-up runs it, `menu::launch`).
    if crate::menu::launch::asked().is_none() {
        crate::menu::play_films::play_films(o, false)?;
    }
    crate::logfile::log_write::log_nul(STARTING, true, 1);
    run_screen(ScreenId::MainMenu, &Entry::new(o))?;
    crate::logfile::log_write::log_nul(FINISHED, true, 1);
    crate::menu::store_player_names::store_player_names(menus, play, &mut |line| {
        crate::logfile::log_write::log_nul(line, true, 1);
    })?;
    crate::gfx::free_fonts::free_fonts(fonts)?;
    crate::saves::list_car_numbers::list_car_numbers_in(menus)?;
    log("menu::run_front_end: done");
    Ok(())
}
