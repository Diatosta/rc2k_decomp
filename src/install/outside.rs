//! What the installation checks ask of the world outside the game's memory, as a trait.

use crate::errors::BadInput;
use crate::files::drive_type::drive_type;
use crate::files::file_exists::file_exists;
use crate::install::normalise_path::{current_directory, normalise_path};
use crate::install::read_record::read_record;
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
        normalise_path(path, to)
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

pub struct Game;

impl Outside for Game {
    unsafe fn file_exists(&mut self, path: *const u8) -> Result<bool, BadInput> {
        file_exists(path)
    }
    unsafe fn drive_type(&mut self, letter: u8) -> u32 {
        unsafe { drive_type(letter) }
    }
    unsafe fn set_directory(&mut self, path: *const u8) -> bool {
        unsafe { SetCurrentDirectoryA(path) != 0 }
    }
    unsafe fn read_record(&mut self) -> Result<(), BadInput> {
        read_record(crate::install::installer::INSTALL_TEXTS.record_path.addr() as *const u8)
    }
    unsafe fn log(&mut self, text: *const u8) {
        unsafe { log_write(text, true, 1) };
    }
}
