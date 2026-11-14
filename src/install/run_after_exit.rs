//! What the installer's pages asked to be done once the game has closed (not after a fatal
//! error): the Start menu items added or removed (shell shortcuts, `install::shortcuts`); then
//! the first of: DirectX's set-up run; the registration program run; or, when installing, the
//! installed folder made current, from the CD the program copied to `.\ral.exe` after a
//! second's sleep, and run. A folder that cannot be made current ends it.
//!
//! One caller, `start`'s way out. Refuses (before anything, where the way could lead there) a
//! game folder too long for "ereg\" in its 0x100 bytes, and a command line's program of 0x100
//! bytes or more.

use crate::errors::{BadInput, Refused};
use crate::install::runs_from_source::{scratch_put, scratch_text};
use rc2k_core::files::paths;

const ROOM: u32 = 0x100;

#[cfg(windows)]
pub static RC2K: &[u8] = b".\\rc2k.exe";
#[cfg(not(windows))]
pub static RC2K: &[u8] = b".\\rc2k";

fn set_cd(path: &[u8]) -> bool {
    std::env::set_current_dir(crate::files::host_path(path)).is_ok()
}

fn run(command: &[u8]) {
    crate::install::shortcuts::run_program(&crate::files::host_path(command));
}
pub fn run_after_exit(
    install: &crate::install::owner::Install,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    let run_vars = &install.run;
    let add_items = run_vars.add_items.get() == 1;
    let remove_items = run_vars.remove_items.get() == 1;
    let run_dxsetup = run_vars.run_dxsetup.get() == 1;
    let installing = install.flags.installing.get();
    let run_registration = run_vars.run_registration.get() == 1;
    if crate::fatal::state::REPORTED.get() == 1 {
        return Ok(());
    }
    // The refusals, before anything.
    let game_dir = install.paths.game_dir.text();
    let game_dir_len = game_dir.len() as u32;
    if !run_dxsetup && run_registration && game_dir_len + 5 + 1 > ROOM {
        return Err(
            BadInput("the game's folder too long for \"ereg\\\" in its 0x100 bytes").into(),
        );
    }
    let from_cd = install.vars.running_from_cd.get();
    if !run_dxsetup
        && !run_registration
        && installing
        && from_cd == 1
        && system.start().command_line.program_len.get() >= ROOM
    {
        return Err(BadInput("a command line's program of 0x100 bytes or more").into());
    }
    // The Start menu items: the standalone's shell shortcuts.
    if add_items {
        crate::install::shortcuts::add_start_menu_items(&crate::files::host_path(
            &install.paths.installed.text(),
        ));
    }
    if remove_items {
        crate::install::shortcuts::remove_start_menu_items();
    }
    if run_dxsetup {
        let other_dir = install.texts.other_dir.text();
        for dir in [&game_dir, &other_dir] {
            if !set_cd(dir) {
                return Ok(());
            }
        }
        run(&install.names.dxsetup.text());
        return Ok(());
    }
    if run_registration {
        // "ereg\eregip.exe" (the script's text after its count): its folder after the
        // game's, its name into the program's name buffer.
        let ereg = install.run.ereg.read();
        let ereg = &ereg[2..];
        let ereg = &ereg[..ereg.iter().position(|&c| c == 0).unwrap_or(ereg.len())];
        let folder = paths::directory(ereg).unwrap_or(&[]);
        let at = game_dir_len as usize;
        install
            .paths
            .game_dir
            .write_at(at, &[folder, &[0]].concat(), "the installer's path")?;
        scratch_put(0, &[paths::file_name(ereg), &[0]].concat())?;
        if !set_cd(&install.paths.game_dir.text()) {
            return Ok(());
        }
        run(&scratch_text()?);
        return Ok(());
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
    // The program and its DLLs from its own folder (not a CD's copy of `ral.exe`).
    crate::install::shortcuts::copy_program();
    crate::system::sleep::sleep(1000);
    run(RC2K);
    Ok(())
}
