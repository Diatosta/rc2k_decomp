//! What the installer's pages asked to be done once the game has closed (not after a fatal
//! error): the Start menu items added or removed (shell shortcuts, `install::shortcuts`); then,
//! when installing, the installed folder made current, from the CD the program copied to
//! `.\ral.exe` after a second's sleep, and run. A folder that cannot be made current ends it.
//!
//! Refuses (before anything, where the way could lead there) a command line's program of 0x100
//! bytes or more.

use crate::errors::{BadInput, Refused};

const ROOM: u32 = 0x100;

#[cfg(windows)]
pub static RC2K: &[u8] = b".\\rc2k.exe";
#[cfg(not(windows))]
pub static RC2K: &[u8] = b".\\rc2k";

fn set_cd(path: &[u8]) -> bool {
    std::env::set_current_dir(crate::files::host_path(path)).is_ok()
}

fn run(command: &[u8]) -> bool {
    crate::install::shortcuts::run_program(&crate::files::host_path(command))
}
/// The installer's own run ends once the installed game was started: nothing of it is to stay
/// behind (a window's last events, the audio, a thread of the library) while the game runs.
/// Not when the game could not be started: the installer's end then goes its usual way.
fn end_with_installed_game(started: bool) {
    if started {
        crate::standalone::finish("the installed game was started", 0);
    }
}

pub fn run_after_exit(
    install: &crate::install::owner::Install,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    let run_vars = &install.run;
    let add_items = run_vars.add_items.get() == 1;
    let remove_items = run_vars.remove_items.get() == 1;
    let installing = install.flags.installing.get();
    if crate::fatal::state::REPORTED.get() == 1 {
        return Ok(());
    }
    // The refusal, before anything.
    let from_cd = install.vars.running_from_cd.get();
    if installing && from_cd == 1 && system.start().command_line.program_len.get() >= ROOM {
        return Err(BadInput("a command line's program of 0x100 bytes or more").into());
    }
    // The Start menu items: the program's shell shortcuts.
    if add_items {
        crate::install::shortcuts::add_start_menu_items(&crate::files::host_path(
            &install.paths.installed.text(),
        ));
    }
    if remove_items {
        crate::install::shortcuts::remove_start_menu_items();
    }
    if !installing {
        return Ok(());
    }
    let normalised = crate::install::normalise_path::normalised_bytes(
        &install.names.source.text(),
        &mut || crate::install::normalise_path::current_directory(),
    )?;
    let Some(installed) = normalised else {
        return Ok(());
    };
    install
        .paths
        .installed
        .write_at(0, &installed, "the installed folder")?;
    if !set_cd(&install.paths.installed.text()) {
        return Ok(());
    }
    // The installed application on macOS (signed again, registered, opened).
    #[cfg(unix)]
    if crate::install::mac_bundle::installing() {
        let opened = crate::install::mac_bundle::finish();
        end_with_installed_game(opened);
    }
    // The program and its DLLs from its own folder (not a CD's copy of `ral.exe`).
    crate::install::shortcuts::copy_program();
    crate::clock::sleep::sleep(1000);
    let started = run(RC2K);
    end_with_installed_game(started);
    Ok(())
}
