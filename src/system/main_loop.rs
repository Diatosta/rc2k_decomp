//! `system::main_loop` (0x415383): what `start` calls once everything is set up: it waits
//! for the window, runs the game (or the installer) and saves on the way out.
//!
//! - A log line "Entering main loop..." (after an empty one).
//! - It sleeps a second at a time until the window is the active one
//!   (`flags.window_active`) and the key thread has run (`clocks.key_ticks`); the quit
//!   switch (`flags.quitting`) at 1 ends the wait and the loop, without `request_quit`.
//! - Then `sound::apply_settings`, `gfx::load_jade` and `menu::build_notice`.
//! - If the installer's script is set up (`pages.script_set_up` is 0xBD) or the program
//!   runs for an update (`run.updating`, the `-u` switch): `install::run_installer`, then
//!   the way out. Else, unless installing, `install::runs_from_source` (its answer is
//!   not used); the installing, updating and from-the-command-line switches are cleared.
//! - `sound::stop_cd_audio`; `files::is_cd_drive` on the base directory's first
//!   character, whose answer this build throws away.
//! - Then `menu::run_front_end`, `sound::stop_music`, `saves::save_game_data`, and unless
//!   the quit switch is 1 by then (which leaves at once) `saves::free_best_times`
//!   (the original also skips the save then: fixed, section 16).
//! - The way out: `request_quit` and "Leaving main loop...".
//!
//! One caller, `start`.
//!
//! Left out, as no Rust code can reach them: the second entry the original's fatal stubs
//! about the CD-ROM took (they jumped back into the loop with the saved stack pointer, to
//! leave it), and the lobby's branch: the standalone has no lobbies (its own network
//! protocol), so `net::set_up` leaves out the original's lobby detection and the
//! "launched from a lobby" flag stays 0.

use crate::errors::Refused;
use crate::framework::log;
use crate::owners::Owners;

/// `InstallerPages.script_set_up` when the installer's script is set up.
const INSTALLER_MARK: u16 = 0xBD;

/// The base directory's first character (`g_baseDirectory`).
fn base_drive() -> u8 {
    crate::files::state::directory_byte(0)
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
pub fn main_loop(o: Owners<'static>) -> Result<(), Refused> {
    let session = &o.system.session;
    let saves = o.saves;
    let gfx = o.gfx;
    let sound = o.sound;
    let install = o.install;
    let menus = o.menus;
    log_line(c"");
    log_line(c"Entering main loop...");
    // Wait for the window to be the active one and the key thread to have run; the quit
    // switch leaves without asking to quit.
    loop {
        crate::system::sleep::sleep(1000);
        if session.flags.quitting.is_set() {
            log_line(c"Leaving main loop...");
            return Ok(());
        }
        if session.flags.window_active.is_set() && session.clocks.key_ticks.is_set() {
            break;
        }
    }
    crate::platform::set_menu_mode_now();
    crate::sound::apply_settings::apply_settings()?;
    crate::gfx::load_jade::load_jade_in_game(gfx)?;
    crate::menu::build_notice::build_notice(menus);
    // The standalone in the download folder installs from it (`install::portable`); and
    // with `-u` (`run.updating`) it uninstalls where it is: its record is in the installed
    // folder, so the uninstall runs from there (the original's ran from the CD, through the
    // update's mark).
    if install.pages.main_menu.items[1].text.get() == INSTALLER_MARK
        || crate::install::portable::is_source_folder()
        || install.run.updating.get() == 1
    {
        crate::install::run_installer::run_installer(install)?;
        return quit_and_leave();
    }
    if install.run.installing.get() != 1 {
        crate::install::runs_from_source::source_against_directory(
            install,
            &mut crate::install::outside::Game {
                install,
                system: o.system,
            },
        )?;
    }
    install.run.installing.set(0);
    install.run.updating.set(0);
    install.run.from_command.set(0);
    crate::sound::stop_cd_audio::stop_cd_audio(sound);
    // Its answer is not read (this build's nine `nop`s).
    crate::files::is_cd_drive::is_cd_drive(base_drive());
    crate::menu::run_front_end::run_front_end(o)?;
    crate::sound::stop_music::stop_music();
    if session.flags.quitting.is_set() {
        // The original leaves here without saving: the window closed (or any quit by the
        // switch) lost the options and best times changed since the last save (section 16);
        // here they are saved.
        crate::saves::save_game_data::save_in_game(saves)?;
        log_line(c"Leaving main loop...");
        return Ok(());
    }
    crate::saves::save_game_data::save_in_game(saves)?;
    crate::saves::free_best_times::free_in_game(saves)?;
    quit_and_leave()
}

/// The main loop, logged on the way in and out.
pub fn main_loop_in_game(o: Owners<'static>) -> Result<(), Refused> {
    log("system::main_loop: runs");
    main_loop(o)?;
    log("system::main_loop: has come back");
    Ok(())
}
