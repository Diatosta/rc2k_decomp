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
//!   `sub_40F826`, and unless the quit switch is 1 by then (which leaves at once)
//!   `saves::save_game_data` and `saves::free_best_times`.
//! - The way out: `request_quit` and "Leaving main loop...".
//!
//! One caller, `start`.
//! Each real call writes a line to hook.log saying which version runs. Refuses what its parts
//! refuse; such a refusal comes with the work half done.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::framework::log;
use crate::pointers::GamePtr;

// ---- game data ----

/// The variables the loop reads and writes, in the order of `Vars`'s fields.
const VARS: [usize; 12] = [
    0, // dword_518390: the stack pointer at entry, 0 before (`session`)
    0, // dword_51837C: the quit switch (`session`)
    0, // dword_518374: the window is the active one (`session`)
    0, // dword_518998: counted up by the key thread (`session`)
    0, // dword_51838C (`session`)
    0, // dword_518080 (`sound::store`)
    0, // word_51D752: 0xBD when the installer's script is set up (`install::vars`)
    0, // g_installing (`install::vars`)
    0, // dword_51B9CC (`install::vars`)
    0, // dword_51B9D0 (`install::vars`)
    0, // g_baseDirectory (`files::state`)
    0, // dword_5AB290: `g_lobbyLaunched` (`net::store`)
];
const ENTERING_RVA: &std::ffi::CStr = c"Entering main loop..."; // "Entering main loop..."
const LEAVING_RVA: &std::ffi::CStr = c"Leaving main loop..."; // "Leaving main loop..."
const INSTALLER_MARK: u16 = 0xBD;

/// Where the loop's variables are: the game's own, or a test's.
#[derive(Clone, Copy)]
pub struct Vars(pub [GamePtr; 12]);

impl Vars {
    fn game() -> Self {
        use crate::session::{CLOCKS, FLAGS};
        let at = |address: usize| GamePtr::new(address as u32);
        let mut vars = VARS.map(|v| {
            if v == 0 {
                GamePtr::new(0)
            } else {
                at(crate::moved::place(v))
            }
        });
        vars[0] = at(FLAGS.main_loop_stack.addr());
        vars[1] = at(FLAGS.quitting.addr());
        vars[2] = at(FLAGS.window_active.addr());
        vars[3] = at(CLOCKS.key_ticks.addr());
        vars[4] = at(FLAGS.f38c.addr());
        vars[5] = at(crate::sound::store::SETTINGS_APPLIED.done.addr());
        vars[6] = at(crate::install::vars::INSTALLER_PAGES.script_set_up.addr());
        vars[7] = at(crate::install::vars::INSTALLER_RUN.installing.addr());
        vars[8] = at(crate::install::vars::INSTALLER_RUN.updating.addr());
        vars[9] = at(crate::install::vars::INSTALLER_RUN.from_command.addr());
        vars[10] = at(crate::files::state::base_directory() as usize);
        vars[11] = at(crate::net::store::NET.lobby_launched.addr());
        Self(vars)
    }

    fn get(&self, n: usize) -> Result<u32, BadInput> {
        self.0[n].read::<u32>("the main loop's variable")
    }

    fn set(&self, n: usize, value: u32) -> Result<(), BadInput> {
        self.0[n].write(value, "the main loop's variable")
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

pub type Outside<'a> = &'a mut dyn FnMut(Call) -> Result<(), Refused>;

/// `stack` is the stack pointer at the loop's entry.
pub fn main_loop(vars: &Vars, stack: u32, outside: Outside) -> Result<(), Refused> {
    {
        let leaving = |outside: Outside| outside(Call::Log(LEAVING_RVA.as_ptr() as usize as u32));
        let quit_and_leave = |outside: Outside| {
            outside(Call::RequestQuit)?;
            outside(Call::Log(LEAVING_RVA.as_ptr() as usize as u32))
        };
        if vars.get(0)? != 0 {
            return quit_and_leave(outside);
        }
        vars.set(0, stack)?;
        outside(Call::Log(0))?;
        outside(Call::Log(ENTERING_RVA.as_ptr() as usize as u32))?;
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
        if vars.0[6].read::<u16>("the installer's mark")? == INSTALLER_MARK
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
        outside(Call::IsCdDrive(
            vars.0[10].read::<u8>("the base directory")?,
        ))?;
        if vars.get(11)? == 1 {
            outside(Call::Sub418DF4)?;
            return quit_and_leave(outside);
        }
        outside(Call::RunFrontEnd)?;
        outside(Call::Sub40F826)?;
        if vars.get(1)? == 1 {
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
    unsafe {
        match call {
            Call::Log(0) => crate::menu::text_entry::log_l1(c"".as_ptr() as *const u8, true),
            Call::Log(text) => crate::menu::text_entry::log_l1(text as *const u8, true),
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
    }
    Ok(())
}

/// The Rust version on the game's own calls. The stack pointer it keeps in
/// `dword_518390` is only the game's way out's (0x4154B5), which no Rust code takes: any
/// value but 0 does, so this frame's.
pub fn main_loop_in_game() -> Result<(), Refused> {
    log("system::main_loop: the Rust version runs");
    let here = 0u32;
    let stack = &raw const here as u32;
    main_loop(&Vars::game(), stack, &mut in_game)?;
    log("system::main_loop: the Rust version has come back");
    Ok(())
}
