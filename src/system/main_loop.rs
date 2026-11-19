//! What `start` calls once everything is set up: it waits for the window (until it is the active
//! one and the key thread has run; the quit switch ends the wait and the loop), runs the game (or
//! the installer when its script is set up or `-u` is given) and saves on the way out:
//! `sound::apply_settings`, `gfx::load_jade`, `menu::build_notice`, `menu::run_front_end`,
//! `sound::stop_music`, `saves::save_game_data` and, unless quitting, `saves::free_best_times`. Log
//! lines "Entering main loop..." and "Leaving main loop...". The save is made when quitting too
//! (fixed: `wiki/Fixed-bugs.md`, "Options lost when the game is closed by its window"). There are
//! no lobbies (the network protocol is our own), so the "launched from a lobby" flag stays 0.

use crate::errors::Refused;
use crate::owners::Owners;
use crate::program_log::log;
use crate::threading::thread_slot::SlotOwners;

/// `InstallerPages.script_set_up` when the installer's script is set up.
const INSTALLER_MARK: u16 = 0xBD;

/// `logfile::log_write` of a line of the game's log.
fn log_line(text: &std::ffi::CStr) {
    crate::logfile::log_write::log_nul(text, true, 1);
}

/// The way out: the program asked to quit, "Leaving main loop...".
fn quit_and_leave(owners: SlotOwners<'static>) -> Result<(), Refused> {
    crate::threading::request_quit::request_quit_logged(owners)?;
    log_line(c"Leaving main loop...");
    Ok(())
}
pub fn main_loop(o: Owners<'static>) -> Result<(), Refused> {
    let session = &o.system.session;
    let saves = o.saves;
    let gfx = o.gfx;
    let install = o.install;
    let menus = o.menus;
    log_line(c"");
    log_line(c"Entering main loop...");
    // Wait for the window to be the active one and the key thread to have run, looked at
    // every 10 ms (the original slept a whole second before looking); the quit switch leaves
    // without asking to quit.
    loop {
        if session.flags.quitting.is_set() {
            log_line(c"Leaving main loop...");
            return Ok(());
        }
        if session.flags.window_active.is_set() && session.key_clock.ticks.is_set() {
            break;
        }
        crate::clock::sleep::sleep(10);
    }
    crate::platform::set_menu_mode_now(gfx, o.system);
    crate::sound::apply_settings::apply_settings(o)?;
    crate::gfx::load_jade::load_jade_in_game(gfx, o.system)?;
    crate::menu::build_notice::build_notice(menus, o.strings);
    // A program in the download folder installs from it (`install::portable`); and with
    // `-u` (`flags.updating`) it uninstalls where it is: its record is in the installed folder, so
    // the uninstall runs from there.
    if install.pages.main_menu.items[1].text.get() == INSTALLER_MARK
        || crate::install::portable::is_source_folder()
        || install.flags.updating.get()
    {
        crate::install::run_installer::run_installer(install, o)?;
        return quit_and_leave(SlotOwners::of(o));
    }
    if !install.flags.installing.get() {
        crate::install::runs_from_source::source_against_directory(
            install,
            &mut crate::install::outside::Game {
                install,
                system: o.system,
            },
        )?;
    }
    install.flags.clear_command();
    crate::menu::run_front_end::run_front_end(o)?;
    crate::sound::stop_music::stop_music(o);
    if session.flags.quitting.is_set() {
        // Saved on this way out too (fixed: `wiki/Fixed-bugs.md`, "Options lost when the game
        // is closed by its window").
        crate::saves::save_game_data::save_in_game(o, saves)?;
        log_line(c"Leaving main loop...");
        return Ok(());
    }
    crate::saves::save_game_data::save_in_game(o, saves)?;
    crate::saves::free_best_times::free_in_game(saves)?;
    quit_and_leave(SlotOwners::of(o))
}

/// The main loop, logged on the way in and out.
pub fn main_loop_in_game(o: Owners<'static>) -> Result<(), Refused> {
    log("system::main_loop: runs");
    main_loop(o)?;
    log("system::main_loop: has come back");
    Ok(())
}
