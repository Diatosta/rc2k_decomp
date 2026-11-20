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
use crate::menu::screen_id::ScreenId;
use crate::menu::screen_loop::{Entry, run_screen};
use crate::owners::Owners;
use crate::program_log::log;

const STARTING: &std::ffi::CStr = c"BEV: Starting GB_Front";
const FINISHED: &std::ffi::CStr = c"BEV: Finished GB_Front";

/// The main menu. A trouble the game can go back from that no item of the main menu caught (a
/// launch's race, the main menu's own set-up) cannot go on at a menu whose set-up it
/// interrupted: the player is told and the game ends as when it is closed.
fn run_main_menu(o: Owners<'static>) -> Result<(), Refused> {
    match crate::error_dialog::recover::guarded(|| run_screen(ScreenId::MainMenu, &Entry::new(o))) {
        Ok(left) => left.map(|_| ()),
        Err(reason) => {
            log(&format!(
                "menu::run_front_end: no menu to go back to: {reason}"
            ));
            crate::error_dialog::fatal(&reason);
            o.system.session.flags.quitting.set(true);
            Ok(())
        }
    }
}

/// What the main menu's item that ran into a trouble the game can go back from does: the
/// screens' leaving flags are cleared (a screen that was left in the middle may have set them),
/// the player is told and chooses the main menu or to quit.
pub(super) fn back_at_main_menu(o: Owners<'static>, reason: &str) {
    log(&format!(
        "menu::run_front_end: back at the main menu: {reason}"
    ));
    clear_screen_state(o.menus);
    if !crate::error_dialog::ask_back(reason) {
        o.system.session.flags.quitting.set(true);
    }
}

/// What a screen that was left in the middle may have set: its leaving flags, the leave
/// question and the event's end.
fn clear_screen_state(menus: &crate::menu::owner::Menus) {
    let screens = &menus.screens;
    let flags = &screens.loop_flags;
    for flag in [
        &flags.event_left,
        &flags.leave_screen,
        &flags.left_with_carry,
        &flags.ask_leave,
    ] {
        flag.set(false);
    }
    screens.event.event_over.set(false);
    screens.flow.asking.set(false);
    screens.draw_work.escape_pressed.set(false);
}

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
    // What the start-up found wrong and mended (the options file's bad values): the screen and
    // the fonts are there now.
    crate::error_dialog::show_pending();
    // The films, at the game's start, unless a race or a replay was asked for on the command line (the
    // main menu's set-up runs it, `menu::launch`).
    if crate::menu::launch::asked().is_none() {
        crate::menu::play_films::play_films(o, false)?;
    }
    crate::logfile::log_write::log_nul(STARTING, true, 1);
    run_main_menu(o)?;
    crate::logfile::log_write::log_nul(FINISHED, true, 1);
    crate::menu::store_player_names::store_player_names(menus, play, &mut |line| {
        crate::logfile::log_write::log_nul(line, true, 1);
    })?;
    crate::gfx::free_fonts::free_fonts(fonts)?;
    crate::saves::list_car_numbers::list_car_numbers_in(menus)?;
    log("menu::run_front_end: done");
    Ok(())
}
