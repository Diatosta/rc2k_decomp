//! `system::read_command_line` (0x410684): takes the switches off the command line.
//!
//! In:  nothing
//! Out: every register preserved
//! The first word is the program (its place and length to `dword_518BA8`, `dword_518BAC`).
//! Of the words after it only those of two characters or more count, by their first two:
//! - `-v`: a message box with the version ("Rally 5.30 (4.8.2000)", a tab, a line break and
//!   the second version text `byte_4E0077`, built in the fatal report's `Text`; the title
//!   is the report's "Rally: Guru Meditation" cut after "Rally"), and the game leaves
//!   through `start`'s exit path (0x401116) without coming back here. The words after
//!   it are not read;
//! - `-u`: `dword_51B9CC` = 1; `-p`: `g_installing` (0x51B9E0) = 1; `-a`: `g_noAudio` = 1;
//! - `-i`: `dword_51B9D0` and `dword_51BA04` = 1, the next two words are copied (without a
//!   NUL) to `byte_95E6F8` and `byte_95E7F8`, and the first characters of the two after
//!   them, less `0`, go to `dword_51BA08` and `dword_51BA0C` (0 for a word that is not
//!   there);
//! - `-c`: the next word's first character: `1` sets `dword_5AB954` (`ignoresecondary`),
//!   `2` sets `dword_5AB950` (`ignoreprimary`).
//!
//! The line ends at a word of no length that starts where the reading stands.
//! One caller, `start`. Refuses a word for `-i` of 0x100 bytes or more.
//! The `-v` part cannot be run in a test: it was looked at by the owner.
//! `start` calls it directly (2026-10-23); the registers above are the original's, for
//! reference.

use crate::audio::store::AUDIO;
use crate::d3d::cards_store::CARDS_3D;
use crate::errors::BadInput;
use crate::install::vars::INSTALLER_PATHS;
use crate::install::vars::INSTALLER_RUN;
use crate::session::Dword;
use crate::system::start_texts::START_TEXTS;
use windows_sys::Win32::System::Environment::GetCommandLineA;
use windows_sys::Win32::UI::WindowsAndMessaging::MessageBoxA;

// ---- game data ----

const EXIT_PATH: usize = 0x401116;

// ---- Rust version ----

/// What the line asks for besides its switches.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Asked {
    Nothing,
    /// `-v`: the version is to be shown and the game left.
    Version,
}

/// The `-v` part: the box, and out through `start`'s exit path. Does not return.
pub unsafe fn show_version_and_leave() -> ! {
    unsafe {
        crate::fatal::state::CAPTION
            .ptr()
            .cast::<u8>()
            .add(5)
            .write(0); // byte_4F6702: its `:`
        let c_text = |at: usize| std::ffi::CStr::from_ptr(at as *const _).to_bytes();
        let mut text = c_text(START_TEXTS.version.addr() + 6).to_vec();
        text.extend(*b"\t\r\n");
        text.extend(c_text(START_TEXTS.second_text.addr()));
        text.push(0);
        let place = crate::fatal::state::TEXT.ptr().cast::<u8>(); // Text
        std::ptr::copy(text.as_ptr(), place, text.len());
        MessageBoxA(
            std::ptr::null_mut(),
            place,
            crate::fatal::state::CAPTION.ptr().cast::<u8>(),
            0x2040,
        );
        // The exit path (0x401116) ends in `ExitProcess`; so does the standalone, here.
        crate::standalone::finish(&format!("the game's way out {EXIT_PATH:#x} was reached"), 0)
    }
}

pub unsafe fn read_command_line(line: *const u8) -> Result<Asked, BadInput> {
    // Converted: `rc2k_core::system::command_line::read`, the whole line read before
    // anything is written (a refused `-i` word now leaves nothing written).
    unsafe {
        let sw = rc2k_core::system::command_line::read(&crate::bridge::Raw::new(line))?;
        COMMAND_LINE
            .program
            .set(line.wrapping_add(sw.program.0) as u32);
        COMMAND_LINE.program_len.set(sw.program.1 as u32);
        for (on, place) in [
            (sw.u, INSTALLER_RUN.updating.addr()),
            (sw.installing, INSTALLER_RUN.installing.addr()),
            (sw.no_audio, AUDIO.no_audio.addr()),
        ] {
            if on {
                (place as *mut u32).write_unaligned(1);
            }
        }
        for i in &sw.install {
            INSTALLER_RUN.from_command.set(1u32);
            INSTALLER_RUN.ignore_room.set(1u32);
            for (place, (start, len)) in [
                INSTALLER_PATHS.game_dir.addr(),
                INSTALLER_PATHS.installed.addr(),
            ]
            .into_iter()
            .zip(i.words)
            {
                std::ptr::copy(line.wrapping_add(start), place as *mut u8, len);
            }
            INSTALLER_RUN.given_mode.set(i.digits[0]);
            INSTALLER_RUN.given_language.set(i.digits[1]);
        }
        if sw.ignore_secondary {
            CARDS_3D.ignore_secondary.set(1u32);
        }
        if sw.ignore_primary {
            CARDS_3D.not_primary.set(1u32);
        }
        Ok(if sw.version {
            Asked::Version
        } else {
            Asked::Nothing
        })
    }
}

/// The Rust version on the program's own command line (`GetCommandLineA`); `-v` shows the
/// version and leaves through `start`'s exit path.
pub unsafe fn read_own_command_line() -> Result<(), BadInput> {
    unsafe {
        if read_command_line(GetCommandLineA())? == Asked::Version {
            show_version_and_leave();
        }
    }
    Ok(())
}

/// The program's name on the command line (IDA 0x518BA8 .. 0x518BB0, the image's layout).
#[repr(C)]
pub struct CommandLine {
    /// 0x518BA8: `dword_518BA8`: the program's name (a pointer into the command line; `system::read_command_line`).
    pub program: Dword,
    /// 0x518BAC: `dword_518BAC`: its length.
    pub program_len: Dword,
}

pub static COMMAND_LINE: CommandLine = CommandLine {
    program: Dword::new(0x0),
    program_len: Dword::new(0x0),
};

impl CommandLine {
    pub const START: usize = 0x518BA8;
    pub const LEN: usize = 0x8;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        &COMMAND_LINE as *const CommandLine as usize
    }
}

#[cfg(test)]
mod command_line_tests {
    use super::*;

    #[test]
    fn commandline_starts_as_the_games() {
        let size = size_of::<CommandLine>();
        assert!(CommandLine::LEN <= size && size <= (CommandLine::LEN).next_multiple_of(4));
        let image = crate::moved::image_bytes(CommandLine::START, CommandLine::LEN);
        let mine = unsafe {
            std::slice::from_raw_parts(CommandLine::home() as *const u8, CommandLine::LEN)
        };
        assert_eq!(mine, &image[..]);
    }
}
