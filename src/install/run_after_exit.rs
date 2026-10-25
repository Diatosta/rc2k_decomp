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
//!   the command line (`dword_518BA8`, `dword_518BAC` bytes, into `byte_94E8B0`) copied
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
use crate::install::vars::INSTALLER_NAMES;
use crate::install::vars::INSTALLER_PATHS;
use crate::install::vars::INSTALLER_RUN;
use crate::system::read_command_line::COMMAND_LINE;

const ROOM: u32 = 0x100;

// The variables.

/// The standalone's program in place of `.\ral.exe` (`install/shortcuts.rs`).
pub static RC2K: [u8; 11] = *b".\\rc2k.exe\0";

/// Where the variables are.
#[derive(Clone, Copy)]
pub struct Vars {
    pub reported: u32,
    pub switches: u32,
    pub game_dir: u32,
    pub installed: u32,
    pub other_dir: u32,
    pub program: u32,
    pub source: u32,
    pub from_cd: u32,
    pub command: u32,
}

impl Vars {
    pub fn game() -> Self {
        Self {
            reported: crate::fatal::state::REPORTED.as_ptr() as u32, // g_fatalReported (a byte)
            switches: INSTALLER_RUN.add_items.addr() as u32,
            game_dir: INSTALLER_PATHS.game_dir.addr() as u32,
            installed: INSTALLER_PATHS.installed.addr() as u32,
            other_dir: crate::install::installer::INSTALL_TEXTS.other_dir.addr() as u32,
            program: crate::files::numbered_path::buffer() as u32, // byte_94E8B0: a program's name (0x100 bytes taken)
            source: INSTALLER_NAMES.source.addr() as u32,
            from_cd: crate::install::store::INSTALL.running_from_cd.addr() as u32,
            command: COMMAND_LINE.program.addr() as u32,
        }
    }
}

fn rd(at: u32) -> u32 {
    unsafe { (at as *const u32).read_volatile() }
}

/// `files::set_current_directory` (0x403105): whether it worked.
fn set_cd(path: u32) -> bool {
    use windows_sys::Win32::System::Environment::SetCurrentDirectoryA;
    unsafe { SetCurrentDirectoryA(path as *const u8) != 0 }
}

/// `system::run_program` (0x410ADC, ebx 1: not waited for): the standalone's own start
/// (`install/shortcuts.rs`).
fn run(command: u32) {
    let command = unsafe { crate::install::shortcuts::c_path(command as *const u8) };
    crate::install::shortcuts::run_program(&command);
}

/// The Rust version on the variables in `v`.
pub fn run_after_exit_with(v: &Vars) -> Result<(), Refused> {
    use crate::files::append_directory::append_directory;
    use crate::files::append_file_name::append_file_name;
    let switch = |n: u32| rd(v.switches + 4 * n);
    if unsafe { (v.reported as *const u8).read_volatile() } == 1 {
        return Ok(());
    }
    // The refusals, before anything.
    let game_dir_len =
        unsafe { crate::strings::text_length::text_length(v.game_dir as *const u8) }?;
    if switch(2) != 1 && switch(5) == 1 && game_dir_len + 5 + 1 > ROOM {
        return Err(
            BadInput("the game's folder too long for \"ereg\\\" in its 0x100 bytes").into(),
        );
    }
    if switch(2) != 1
        && switch(5) != 1
        && switch(3) == 1
        && rd(v.from_cd) == 1
        && rd(v.command + 4) >= ROOM
    {
        return Err(BadInput("a command line's program of 0x100 bytes or more").into());
    }
    // install::add_start_menu_items, install::remove_start_menu_items (DDE in the
    // original): the standalone's shell shortcuts.
    if switch(0) == 1 {
        use crate::install::shortcuts::{add_start_menu_items, c_path};
        unsafe { add_start_menu_items(&c_path(INSTALLER_PATHS.installed.addr() as *const u8)) };
    }
    if switch(1) == 1 {
        crate::install::shortcuts::remove_start_menu_items();
    }
    if switch(2) == 1 {
        for dir in [v.game_dir, v.other_dir] {
            if !set_cd(dir) {
                return Ok(());
            }
        }
        run(INSTALLER_NAMES.dxsetup.addr() as u32);
        return Ok(());
    }
    if switch(5) == 1 {
        unsafe {
            // "ereg\eregip.exe" (the script's text after its count).
            let ereg = (INSTALLER_RUN.ereg.addr() + 2) as *const u8;
            let (cursor, _) = append_directory(ereg, (v.game_dir + game_dir_len) as *mut u8)?;
            cursor.write(0);
            let cursor = append_file_name(ereg, v.program as *mut u8)?;
            cursor.write(0);
        }
        if !set_cd(v.game_dir) {
            return Ok(());
        }
        run(v.program);
        return Ok(());
    }
    if switch(3) != 1 {
        return Ok(());
    }
    if !crate::install::installer::normalise(v.source, v.installed)? || !set_cd(v.installed) {
        return Ok(());
    }
    // The program and its DLLs from its own folder (not a CD's copy of `ral.exe`).
    crate::install::shortcuts::copy_program();
    crate::system::sleep::sleep(1000);
    run(RC2K.as_ptr() as u32);
    Ok(())
}

/// After the game: the shortcuts and the programs run, as the switches say.
pub fn run_after_exit() -> Result<(), Refused> {
    run_after_exit_with(&Vars::game())
}
