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
//! - If `dword_5AB290` is 1: `sub_418DF4`, then the way out. Else `menu::run_front_end`,
//!   `sub_40F826`, `saves::save_game_data`, and unless the quit switch is 1 by then (which
//!   leaves at once) `saves::free_best_times` (the original also skips the save then: fixed,
//!   section 16).
//! - The way out: `request_quit` and "Leaving main loop...".
//!
//! One caller, `start`.
//! Each real call writes a line to hook.log saying which version runs. Refuses what its parts
//! refuse; such a refusal comes with the work half done.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::framework::log;
use crate::session::{Cell32, Word};

// ---- game data ----

const ENTERING_RVA: &std::ffi::CStr = c"Entering main loop..."; // "Entering main loop..."
const LEAVING_RVA: &std::ffi::CStr = c"Leaving main loop..."; // "Leaving main loop..."
const INSTALLER_MARK: u16 = 0xBD;

/// The variables the loop reads and writes, by `Vars`'s index: 0 `g_mainLoopStack`
/// (dword_518390, the stack pointer at entry, 0 before), 1 the quit switch (dword_51837C),
/// 2 the window is the active one (dword_518374), 3 the key thread's turns (dword_518998),
/// 4 dword_51838C, 5 dword_518080 (`sound::store`), 7 `g_installing`, 8 dword_51B9CC,
/// 9 dword_51B9D0 (`install::vars`), 11 `g_lobbyLaunched` (dword_5AB290, `net::store`).
/// word_51D752 (0xBD when the installer's script is set up) and the base directory's first
/// character are read apart.
pub struct Vars {
    cells: [&'static dyn Cell32; 12],
    installer_mark: &'static Word,
}

impl Vars {
    fn game() -> Self {
        use crate::install::vars::{INSTALLER_PAGES, INSTALLER_RUN};
        use crate::session::{CLOCKS, FLAGS};
        Self {
            cells: [
                &FLAGS.main_loop_stack,
                &FLAGS.quitting,
                &FLAGS.window_active,
                &CLOCKS.key_ticks,
                &FLAGS.f38c,
                &crate::sound::store::SETTINGS_APPLIED.done,
                &FLAGS.f38c, // not read: word_51D752 is `installer_mark`
                &INSTALLER_RUN.installing,
                &INSTALLER_RUN.updating,
                &INSTALLER_RUN.from_command,
                &FLAGS.f38c, // not read: the base directory
                &crate::net::store::NET.lobby_launched,
            ],
            installer_mark: &INSTALLER_PAGES.script_set_up,
        }
    }

    fn get(&self, n: usize) -> Result<u32, BadInput> {
        Ok(self.cells[n].bits())
    }

    fn set(&self, n: usize, value: u32) -> Result<(), BadInput> {
        self.cells[n].set_bits(value);
        Ok(())
    }
}

// ---- Rust version ----

/// What the loop calls, in the order of the test's stand-ins.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Call {
    /// A line of the game's log: the text's address, or 0 for the empty line.
    Log(u32),
    Sleep(u32),
    Sub404823,
    Sub40EFF3,
    Sub4075F3,
    Sub4165F1,
    RunsFromSource,
    Installer,
    Sub40FCAA,
    /// `files::is_cd_drive` on this character.
    IsCdDrive(u8),
    Sub418DF4,
    RunFrontEnd,
    Sub40F826,
    SaveGameData,
    FreeBestTimes,
    RequestQuit,
}

/// The base directory's first character (`g_baseDirectory`).
fn base_drive() -> u8 {
    // SAFETY: `DIRECTORIES` is a Rust static of 0x200 bytes; the main thread alone writes
    // it, at start-up, as the original's did.
    unsafe { *crate::files::state::base_directory() }
}

pub type Outside<'a> = &'a mut dyn FnMut(Call) -> Result<(), Refused>;

/// `stack` is the stack pointer at the loop's entry.
pub fn main_loop(vars: &Vars, stack: u32, outside: Outside) -> Result<(), Refused> {
    {
        let leaving = |outside: Outside| {
            outside(Call::Log(crate::pointers::game_address(
                LEAVING_RVA.as_ptr(),
            )))
        };
        let quit_and_leave = |outside: Outside| {
            outside(Call::RequestQuit)?;
            outside(Call::Log(crate::pointers::game_address(
                LEAVING_RVA.as_ptr(),
            )))
        };
        if vars.get(0)? != 0 {
            return quit_and_leave(outside);
        }
        vars.set(0, stack)?;
        outside(Call::Log(0))?;
        outside(Call::Log(crate::pointers::game_address(
            ENTERING_RVA.as_ptr(),
        )))?;
        loop {
            outside(Call::Sleep(1000))?;
            if vars.get(1)? == 1 {
                return leaving(outside);
            }
            if vars.get(2)? != 0 && vars.get(3)? != 0 {
                break;
            }
        }
        vars.set(4, 0)?;
        vars.set(5, 0)?;
        outside(Call::Sub404823)?;
        outside(Call::Sub40EFF3)?;
        outside(Call::Sub4075F3)?;
        outside(Call::Sub4165F1)?;
        // The standalone in the download folder installs from it (`install::portable`); and
        // with `-u` (`dword_51B9CC`) it uninstalls where it is: its record is in the
        // installed folder, so the uninstall runs from there (the original's ran from the
        // CD, through the update's mark).
        if vars.installer_mark.get() == INSTALLER_MARK
            || crate::install::portable::is_source_folder()
            || vars.get(8)? == 1
        {
            outside(Call::Installer)?;
            return quit_and_leave(outside);
        }
        if vars.get(7)? != 1 {
            outside(Call::RunsFromSource)?;
        }
        vars.set(7, 0)?;
        vars.set(8, 0)?;
        vars.set(9, 0)?;
        outside(Call::Sub40FCAA)?;
        outside(Call::IsCdDrive(base_drive()))?;
        if vars.get(11)? == 1 {
            outside(Call::Sub418DF4)?;
            return quit_and_leave(outside);
        }
        outside(Call::RunFrontEnd)?;
        outside(Call::Sub40F826)?;
        if vars.get(1)? == 1 {
            // The original leaves here without saving: the window closed (or any quit by
            // the switch) lost the options and best times changed since the last save
            // (section 16, fixed 2026-10-29).
            outside(Call::SaveGameData)?;
            return leaving(outside);
        }
        outside(Call::SaveGameData)?;
        outside(Call::FreeBestTimes)?;
        quit_and_leave(outside)
    }
}

/// The calls: the Rust versions, those still taking registers by their address (a
/// local register set, their scratch: none of them reads one).
fn in_game(call: Call) -> Result<(), Refused> {
    match call {
        Call::Log(0) => crate::logfile::log_write::log_nul(c"", true, 1),
        Call::Log(text) => crate::logfile::log_write::log_at(text, true, 1),
        Call::Sleep(milliseconds) => crate::system::sleep::sleep(milliseconds),
        Call::Sub404823 => {
            crate::platform::set_menu_mode_now();
        }
        Call::Sub40EFF3 => crate::sound::apply_settings::apply_settings()?,
        Call::Sub4075F3 => crate::gfx::load_jade::load_jade_in_game()?,
        Call::Sub4165F1 => crate::menu::build_notice::build_notice(),
        Call::RunsFromSource => {
            crate::install::runs_from_source::source_against_directory(
                &mut crate::install::outside::Game,
            )?;
        }
        Call::Installer => crate::install::run_installer::run_installer()?,
        Call::Sub40FCAA => crate::sound::stop_cd_audio::stop_cd_audio(),
        // Its answer is not read.
        Call::IsCdDrive(drive) => {
            crate::files::is_cd_drive::is_cd_drive(drive);
        }
        // `net::lobby_launched_game`'s Rust version (a case it refuses through its hook).
        Call::Sub418DF4 => crate::net::lobby_launched_game::lobby_launched_game()?,
        Call::RunFrontEnd => crate::menu::run_front_end::run_front_end()?,
        Call::Sub40F826 => crate::sound::stop_music::stop_music(),
        Call::SaveGameData => {
            crate::saves::save_game_data::save_in_game()?;
        }
        Call::FreeBestTimes => crate::saves::free_best_times::free_in_game()?,
        Call::RequestQuit => crate::threading::request_quit::request_quit_logged()?,
    }
    Ok(())
}

/// The Rust version on the game's own calls. The stack pointer it keeps in
/// `dword_518390` is only the game's way out's (0x4154B5), which no Rust code takes: any
/// value but 0 does, so this frame's.
pub fn main_loop_in_game() -> Result<(), Refused> {
    log("system::main_loop: the Rust version runs");
    let here = 0u32;
    let stack = crate::pointers::game_address(&raw const here);
    main_loop(&Vars::game(), stack, &mut in_game)?;
    log("system::main_loop: the Rust version has come back");
    Ok(())
}
