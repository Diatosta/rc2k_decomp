//! What the installation checks ask of the world outside the game's memory, as a trait:
//! the Rust versions for the game, a test's answers for a self-test.

use crate::errors::BadInput;
use crate::files::drive_type::drive_type;
use crate::files::file_exists::file_exists;
use crate::framework::{Regs, game_addr, rva};
use crate::install::normalise_path::{current_directory, normalise_path};
use crate::install::read_record::read_record;
use crate::install::stand_ins::act;
use crate::logfile::log_write::log_write;
use windows_sys::Win32::System::Environment::SetCurrentDirectoryA;

pub trait Outside {
    unsafe fn file_exists(&mut self, path: *const u8) -> Result<bool, BadInput>;
    unsafe fn drive_type(&mut self, letter: u8) -> u32;
    /// `files::set_current_directory` (0x403105): whether it worked.
    unsafe fn set_directory(&mut self, path: *const u8) -> bool;
    /// `install::read_record` on `var\ral.zog`.
    unsafe fn read_record(&mut self) -> Result<(), BadInput>;
    /// A line for the game's log.
    unsafe fn log(&mut self, text: *const u8);

    /// `install::read_registry`.
    unsafe fn read_registry(&mut self) {
        unsafe { crate::install::read_registry::read_registry_here() };
    }
    // The same for the game and for a test:
    unsafe fn normalise_path(&mut self, path: *const u8, to: *mut u8) -> Result<bool, BadInput> {
        unsafe { normalise_path(path, to) }
    }
    /// `files::current_directory` (0x4030D1): the current directory with a `\` after it,
    /// written at `to`; whether it could be had.
    unsafe fn current_directory(&mut self, to: *mut u8) -> Result<bool, BadInput> {
        unsafe {
            let Some(mut directory) = current_directory()? else {
                return Ok(false);
            };
            if !matches!(directory.last(), Some(b'\\' | b'/')) {
                directory.push(b'\\');
            }
            directory.push(0);
            std::ptr::copy(directory.as_ptr(), to, directory.len());
            Ok(true)
        }
    }
}

const RECORD_PATH_RVA: usize = rva(0x5295C0); // "var\ral.zog"

pub struct Game;

impl Outside for Game {
    unsafe fn file_exists(&mut self, path: *const u8) -> Result<bool, BadInput> {
        unsafe { file_exists(path) }
    }
    unsafe fn drive_type(&mut self, letter: u8) -> u32 {
        unsafe { drive_type(letter) }
    }
    unsafe fn set_directory(&mut self, path: *const u8) -> bool {
        unsafe { SetCurrentDirectoryA(path) != 0 }
    }
    unsafe fn read_record(&mut self) -> Result<(), BadInput> {
        unsafe { read_record(game_addr(RECORD_PATH_RVA) as *const u8) }
    }
    unsafe fn log(&mut self, text: *const u8) {
        unsafe { log_write(text, true, 1) };
    }
}

/// The places a self-test's stand-ins have.
pub const FILE_EXISTS: u32 = 0;
pub const DRIVE_TYPE: u32 = 1;
pub const SET_DIRECTORY: u32 = 2;
pub const READ_RECORD: u32 = 3;
pub const LOG: u32 = 4;
pub const READ_REGISTRY: u32 = 5;

/// The test's answers, through the same function the original's copy gets them from.
pub struct Tested;

impl Outside for Tested {
    unsafe fn file_exists(&mut self, path: *const u8) -> Result<bool, BadInput> {
        let mut regs = Regs {
            eax: path as u32,
            ..Regs::default()
        };
        act(FILE_EXISTS, &mut regs);
        Ok(!regs.carry())
    }
    unsafe fn drive_type(&mut self, letter: u8) -> u32 {
        let mut regs = Regs {
            eax: letter as u32,
            ..Regs::default()
        };
        act(DRIVE_TYPE, &mut regs);
        regs.eax
    }
    unsafe fn set_directory(&mut self, path: *const u8) -> bool {
        let mut regs = Regs {
            edx: path as u32,
            ..Regs::default()
        };
        act(SET_DIRECTORY, &mut regs);
        !regs.carry()
    }
    unsafe fn read_record(&mut self) -> Result<(), BadInput> {
        act(READ_RECORD, &mut Regs::default());
        Ok(())
    }
    unsafe fn log(&mut self, text: *const u8) {
        let mut regs = Regs {
            eax: text as u32,
            ..Regs::default()
        };
        act(LOG, &mut regs);
    }
    unsafe fn read_registry(&mut self) {
        act(READ_REGISTRY, &mut Regs::default());
    }
}
