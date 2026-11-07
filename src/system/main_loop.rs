//! `system::main_loop` (0x415383): what `start` calls once everything is set up: it waits
//! for the window, runs the game (or the installer) and saves on the way out.
//!
//! In:  nothing
//! Out: nothing (its one caller looks at no register)
//! - If `dword_518390` is not 0 the loop is being entered a second time, by the way out
//!   at 0x4154B5 (`mov esp, dword_518390; jmp` here, taken by two fatal stubs about the
//!   CD-ROM): `threading::request_quit`, the log line "Leaving main loop...", and back to
//!   `start`. Else the stack pointer is kept there.
//! - An empty log line and "Entering main loop...".
//! - It sleeps a second at a time until the window is the active one (`dword_518374`)
//!   and the key thread has run (`dword_518998`); the quit switch (`dword_51837C`) at 1
//!   ends the wait and the loop, without `request_quit`.
//! - `dword_51838C` and `dword_518080` = 0; `sub_404823`, `sub_40EFF3`, `sub_4075F3`,
//!   `sub_4165F1` (not ported).
//! - If the installer's script is set up (`word_51D752` is 0xBD): the installer
//!   `sub_412DE0`, then the way out. Else, unless installing, `install::runs_from_source`
//!   (its answer is not used); `g_installing`, `dword_51B9CC` and `dword_51B9D0` = 0.
//! - `sub_40FCAA`; `files::is_cd_drive` (0x403310) on the base directory's first
//!   character, whose answer this build's nine `nop`s throw away.
//! - If `g_lobbyLaunched` (dword_5AB290) is 1: `net::lobby_launched_game` (0x418DF4), then
//!   the way out. Else `menu::run_front_end`, `sub_40F826`, `saves::save_game_data`, and
//!   unless the quit switch is 1 by then (which leaves at once) `saves::free_best_times`
//!   (the original also skips the save then: fixed, section 16).
//! - The way out: `request_quit` and "Leaving main loop...".
//!
//! One caller, `start`.
//!
//! Left out, as no Rust code can reach them: the second entry (only the original's way out
//! at 0x4154B5 jumps back in), so `g_mainLoopStack` is not kept; `dword_51838C` and
//! `dword_518080` made 0, which nothing reads; and the lobby's branch: the standalone has no lobbies (its own network protocol), so `net::set_up` leaves out the
//! original's lobby detection (0x418E23) and `g_lobbyLaunched` stays 0.

use crate::errors::Refused;
use crate::framework::log;
use crate::system::owner::{Session, system};

/// `word_51D752` when the installer's script is set up.
const INSTALLER_MARK: u16 = 0xBD;

/// The base directory's first character (`g_baseDirectory`).
fn base_drive() -> u8 {
    crate::files::state::base_directory_place()
        .read("the base directory")
        .unwrap_or_else(|e| crate::fatal::raise::end("the base directory", e))
}

/// `logfile::log_write` of a line of the game's log.
fn log_line(text: &std::ffi::CStr) {
    crate::logfile::log_write::log_nul(text, true, 1);
}

/// The way out: the program asked to quit, "Leaving main loop...".
fn quit_and_leave() -> Result<(), Refused> {
    crate::threading::request_quit::request_quit_logged()?;
    log_line(c"Leaving main loop...");
    Ok(())
}

/// The Rust version.
pub fn main_loop(session: &Session) -> Result<(), Refused> {
    let install = crate::install::owner::install();
    let menus = crate::menu::owner::menus();
    log_line(c"");
    log_line(c"Entering main loop...");
    // Wait for the window to be the active one and the key thread to have run; the quit
    // switch leaves without asking to quit.
    loop {
        crate::system::sleep::sleep(1000);
        if session.flags.quitting.get() == 1 {
            log_line(c"Leaving main loop...");
            return Ok(());
        }
        if session.flags.window_active.get() != 0 && session.clocks.key_ticks.get() != 0 {
            break;
        }
    }
    crate::platform::set_menu_mode_now();
    crate::sound::apply_settings::apply_settings()?;
    crate::gfx::load_jade::load_jade_in_game()?;
    crate::menu::build_notice::build_notice(menus);
    // The standalone in the download folder installs from it (`install::portable`); and
    // with `-u` (`dword_51B9CC`) it uninstalls where it is: its record is in the installed
    // folder, so the uninstall runs from there (the original's ran from the CD, through the
    // update's mark).
    if install.pages.script_set_up.get() == INSTALLER_MARK
        || crate::install::portable::is_source_folder()
        || install.run.updating.get() == 1
    {
        crate::install::run_installer::run_installer(install)?;
        return quit_and_leave();
    }
    if install.run.installing.get() != 1 {
        crate::install::runs_from_source::source_against_directory(
            install,
            &mut crate::install::outside::Game,
        )?;
    }
    install.run.installing.set(0);
    install.run.updating.set(0);
    install.run.from_command.set(0);
    crate::sound::stop_cd_audio::stop_cd_audio();
    // Its answer is not read (this build's nine `nop`s).
    crate::files::is_cd_drive::is_cd_drive(base_drive());
    crate::menu::run_front_end::run_front_end(menus)?;
    crate::sound::stop_music::stop_music();
    if session.flags.quitting.get() == 1 {
        // The original leaves here without saving: the window closed (or any quit by the
        // switch) lost the options and best times changed since the last save (section 16,
        // fixed 2026-10-29).
        crate::saves::save_game_data::save_in_game()?;
        log_line(c"Leaving main loop...");
        return Ok(());
    }
    crate::saves::save_game_data::save_in_game()?;
    crate::saves::free_best_times::free_in_game()?;
    quit_and_leave()
}

/// The Rust version, logged on the way in and out.
pub fn main_loop_in_game() -> Result<(), Refused> {
    log("system::main_loop: the Rust version runs");
    main_loop(&system().session)?;
    log("system::main_loop: the Rust version has come back");
    Ok(())
}
