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

use crate::errors::BadInput;
use crate::framework::{Regs, game_addr, game_write, rva};
use windows_sys::Win32::System::Environment::GetCommandLineA;
use windows_sys::Win32::UI::WindowsAndMessaging::MessageBoxA;

// ---- game data ----

const PROGRAM_RVA: usize = rva(0x518BA8); // dword_518BA8
const PROGRAM_LEN_RVA: usize = rva(0x518BAC); // dword_518BAC
const SWITCH_U_RVA: usize = rva(0x51B9CC); // dword_51B9CC
const INSTALLING_RVA: usize = rva(0x51B9E0); // g_installing
const SWITCH_I_RVA: usize = rva(0x51B9D0); // dword_51B9D0
const SWITCH_I_TOO_RVA: usize = rva(0x51BA04); // dword_51BA04
const I_FIRST_RVA: usize = rva(0x95E6F8); // byte_95E6F8
const I_SECOND_RVA: usize = rva(0x95E7F8); // byte_95E7F8
const I_THIRD_RVA: usize = rva(0x51BA08); // dword_51BA08
const I_FOURTH_RVA: usize = rva(0x51BA0C); // dword_51BA0C
const IGNORE_SECONDARY_RVA: usize = rva(0x5AB954); // dword_5AB954
const IGNORE_PRIMARY_RVA: usize = rva(0x5AB950); // dword_5AB950
const NO_AUDIO_RVA: usize = rva(0x4F82B4); // g_noAudio
const VERSION_RVA: usize = rva(0x4E0000); // "$VER: Rally 5.30 (4.8.2000)"
const SECOND_TEXT_RVA: usize = rva(0x4E0077); // byte_4E0077
const REPORT_TEXT_RVA: usize = rva(0x9526F8); // Text
const CAPTION_RVA: usize = rva(0x4F66FD); // "Rally: Guru Meditation"
const CAPTION_END_RVA: usize = rva(0x4F6702); // byte_4F6702: its `:`
const EXIT_PATH_RVA: usize = rva(0x401116);

// ---- Rust version ----

/// What the line asks for besides its switches.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Asked {
    Nothing,
    /// `-v`: the version is to be shown and the game left.
    Version,
}

/// The `-v` part: the box, and out through `start`'s exit path. Does not return.
pub unsafe fn show_version_and_leave(regs: &Regs) -> ! {
    unsafe {
        game_write(CAPTION_END_RVA, 0u8);
        let c_text = |at: usize| std::ffi::CStr::from_ptr(game_addr(at) as *const _).to_bytes();
        let mut text = c_text(VERSION_RVA + 6).to_vec();
        text.extend(*b"\t\r\n");
        text.extend(c_text(SECOND_TEXT_RVA));
        text.push(0);
        let place = game_addr(REPORT_TEXT_RVA) as *mut u8;
        std::ptr::copy(text.as_ptr(), place, text.len());
        MessageBoxA(
            std::ptr::null_mut(),
            place,
            game_addr(CAPTION_RVA) as *const u8,
            0x2040,
        );
        let mut regs = *regs;
        crate::calls::call(EXIT_PATH_RVA, &mut regs);
        // The exit path ends in `ExitProcess`.
        std::process::abort()
    }
}

pub unsafe fn read_command_line(line: *const u8) -> Result<Asked, BadInput> {
    // Converted: `rc2k_core::system::command_line::read`, the whole line read before
    // anything is written (a refused `-i` word now leaves nothing written).
    unsafe {
        let sw = rc2k_core::system::command_line::read(&crate::bridge::Raw::new(line))?;
        game_write(PROGRAM_RVA, line.wrapping_add(sw.program.0) as u32);
        game_write(PROGRAM_LEN_RVA, sw.program.1 as u32);
        for (on, place) in [
            (sw.u, SWITCH_U_RVA),
            (sw.installing, INSTALLING_RVA),
            (sw.no_audio, NO_AUDIO_RVA),
        ] {
            if on {
                game_write(place, 1u32);
            }
        }
        for i in &sw.install {
            game_write(SWITCH_I_RVA, 1u32);
            game_write(SWITCH_I_TOO_RVA, 1u32);
            for (place, (start, len)) in [I_FIRST_RVA, I_SECOND_RVA].into_iter().zip(i.words) {
                std::ptr::copy(line.wrapping_add(start), game_addr(place) as *mut u8, len);
            }
            game_write(I_THIRD_RVA, i.digits[0]);
            game_write(I_FOURTH_RVA, i.digits[1]);
        }
        if sw.ignore_secondary {
            game_write(IGNORE_SECONDARY_RVA, 1u32);
        }
        if sw.ignore_primary {
            game_write(IGNORE_PRIMARY_RVA, 1u32);
        }
        Ok(if sw.version {
            Asked::Version
        } else {
            Asked::Nothing
        })
    }
}

/// The Rust version on the program's own command line (`GetCommandLineA`); `-v` shows the
/// version and leaves through `start`'s exit path with `regs`.
pub unsafe fn read_own_command_line(regs: &Regs) -> Result<(), BadInput> {
    unsafe {
        if read_command_line(GetCommandLineA())? == Asked::Version {
            show_version_and_leave(regs);
        }
    }
    Ok(())
}
