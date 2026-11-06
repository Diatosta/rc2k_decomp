//! `install::run_after_exit` (0x412FA2): what the installer's pages asked to be done once
//! the game has closed (not after a fatal error, `g_fatalReported` 1):
//! - `dword_51B9D4` 1: the Start menu's items added (`sub_413E42`, DDE to the Program
//!   Manager: "Rally", "Uninstall Rally", "View Readme"); `dword_51B9D8` 1: removed
//!   (`sub_4140C3`, DDE);
//! - then the first of: `dword_51B9DC` 1: DirectX's set-up run (into the game's folder
//!   `byte_95E6F8`, then `unk_5295B5`; `sub_410ADC` "redist\directx\dxsetup.exe", not
//!   waited for); `dword_51B9E8` 1: the registration program run (its folder, the game's
//!   and "ereg\", made current; "eregip.exe" into `byte_94E8B0`); installing
//!   (`g_installing` 1): the installed folder (`g_installSource` normalised into
//!   `byte_95E7F8`) made current, from the CD (`g_runningFromCd` 1) the program named on
//!   the command line (`dword_518BA8`, `dword_518BAC` bytes, into `byte_94E8B0`; the standalone
//!   copies its own folder's program instead, so only the length is kept) copied
//!   to ".\ral.exe" (`files::copy_file`) and a second's sleep, and ".\ral.exe" run.
//!
//! A folder that cannot be made current ends it.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! One caller, `start`'s way out. Refuses (before
//! anything, where the way could lead there) a game folder too long for "ereg\" in its
//! 0x100 bytes, and a command line's program of 0x100 bytes or more.
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

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

/// The Rust version (see the header).
pub fn run_after_exit(g: &'static crate::install::owner::Install) -> Result<(), Refused> {
    let run_vars = &g.run;
    // `g_addStartItems` .. `g_runRegistration`.
    let switch = |n: u32| {
        [
            &run_vars.add_items,
            &run_vars.remove_items,
            &run_vars.run_dxsetup,
            &run_vars.installing,
            &run_vars.offer_registration,
            &run_vars.run_registration,
        ][n as usize]
            .get()
    };
    if crate::fatal::state::REPORTED.get() == 1 {
        return Ok(());
    }
    // The refusals, before anything.
    let game_dir = g.paths.game_dir.text();
    let game_dir_len = game_dir.len() as u32;
    if switch(2) != 1 && switch(5) == 1 && game_dir_len + 5 + 1 > ROOM {
        return Err(
            BadInput("the game's folder too long for \"ereg\\\" in its 0x100 bytes").into(),
        );
    }
    let from_cd = g.vars.running_from_cd.get();
    if switch(2) != 1
        && switch(5) != 1
        && switch(3) == 1
        && from_cd == 1
        && crate::system::owner::system()
            .start
            .command_line
            .program_len
            .get()
            >= ROOM
    {
        return Err(BadInput("a command line's program of 0x100 bytes or more").into());
    }
    // install::add_start_menu_items, install::remove_start_menu_items (DDE in the
    // original): the standalone's shell shortcuts.
    if switch(0) == 1 {
        crate::install::shortcuts::add_start_menu_items(&crate::files::host_path(
            &g.paths.installed.text(),
        ));
    }
    if switch(1) == 1 {
        crate::install::shortcuts::remove_start_menu_items();
    }
    if switch(2) == 1 {
        let other_dir = g.texts.other_dir.text();
        for dir in [&game_dir, &other_dir] {
            if !set_cd(dir) {
                return Ok(());
            }
        }
        run(&g.names.dxsetup.text());
        return Ok(());
    }
    if switch(5) == 1 {
        // "ereg\eregip.exe" (the script's text after its count): its folder after the
        // game's, its name into the program's buffer (`byte_94E8B0`).
        let ereg = g.run.ereg.read();
        let ereg = &ereg[2..];
        let ereg = &ereg[..ereg.iter().position(|&c| c == 0).unwrap_or(ereg.len())];
        let folder = paths::directory(ereg).unwrap_or(&[]);
        let at = game_dir_len as usize;
        g.paths
            .game_dir
            .write_at(at, &[folder, &[0]].concat(), "the installer's path")?;
        scratch_put(0, &[paths::file_name(ereg), &[0]].concat())?;
        if !set_cd(&g.paths.game_dir.text()) {
            return Ok(());
        }
        run(&scratch_text()?);
        return Ok(());
    }
    if switch(3) != 1 {
        return Ok(());
    }
    let normalised =
        crate::install::normalise_path::normalised_bytes(&g.names.source.text(), &mut || {
            crate::install::normalise_path::current_directory()
        })?;
    let Some(installed) = normalised else {
        return Ok(());
    };
    g.paths
        .installed
        .write_at(0, &installed, "the installed folder")?;
    if !set_cd(&g.paths.installed.text()) {
        return Ok(());
    }
    // The program and its DLLs from its own folder (not a CD's copy of `ral.exe`).
    crate::install::shortcuts::copy_program();
    crate::system::sleep::sleep(1000);
    run(RC2K);
    Ok(())
}
