//! `install::check_at_start` (0x416477): finds the game's data when the game is started
//! from somewhere that is not an installation.
//!
//! In:  nothing
//! Out: every register preserved; or it does not return (fatal error 0x90002103)
//! Nothing if the installer is running (`g_installing`) or `var\ral.zog` is there: the
//! usual case. Otherwise `g_noAudio` = 1, and nothing more if `files\d\gfx\jade.bun` is
//! there. Otherwise:
//! - `read_registry`; if an installation is recorded (status 1, a source) and
//!   `<source>ral.exe` exists (the source through `normalise_path` into `byte_94E8B0`),
//!   the installer's script is set up for an update: the text "patch6:  (4.8.2000)" and a
//!   line feed at `byte_A8C054`, and in the entry at `off_51D728` the bytes 5 and 0x0C at
//!   +0x18, the word 0x1006 at +0x1A and the word 0xBD at +0x2A (`word_51D752`, which
//!   sends the main loop into the installer);
//! - then, either way, the drives a .. z: the first CD-ROM on which `x:\ral.exe` and
//!   `x:\files\d\gfx\jade.bun` exist and whose root can be made the current directory
//!   ends the search with `g_runningFromCd` (0x529DB0) = 1. If there is none: "Update
//!   cannot find RC CD-ROM" in the log and the fatal error.
//!
//! One caller, `start`. This is not the copy protection's CD check (that one is in the
//! main loop and `nop`ped out in this build).
//! Refuses a scratch path that is not a text when the source cannot be put there.
//! Self-test: a redirected copy in worlds of the test's (`install/check_test.rs`); the
//! copy's jump to the fatal stub lands back in the test.

use crate::errors::{BadInput, FatalError, Refused};
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::install::outside::{Game, Outside};
use crate::install::read_registry::{SOURCE_RVA, STATUS_RVA};
use crate::install::runs_from_source::SCRATCH_PATH_RVA;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, state_entry};

// ---- game data ----

pub const INSTALLING_RVA: usize = rva(0x51B9E0); // g_installing
pub const RECORD_RVA: usize = rva(0x5295C0); // "var\ral.zog"
const NO_AUDIO_RVA: usize = rva(0x4F82B4); // g_noAudio
const DATA_FILE_RVA: usize = rva(0x4F66C8); // "files\d\gfx\jade.bun"
const PROGRAM_RVA: usize = rva(0x51BE40); // "ral.exe" (in ".\ral.exe")
const VERSION_RVA: usize = rva(0x4E0000); // "$VER: Rally 5.30 (4.8.2000)"
const PATCH_TEXT_RVA: usize = rva(0xA8C054); // byte_A8C054
/// off_51D728: the entry of the installer's script that the checks set up. +0x18 and
/// +0x19 two bytes, +0x1A a string's id, +0x2A (`word_51D752`) 0xBD = the installer runs.
pub const SCRIPT_ENTRY_RVA: usize = rva(0x51D728);
const RUNNING_FROM_CD_RVA: usize = rva(0x529DB0); // g_runningFromCd
const PATH_ROOM: usize = 0x100;

/// What both checks write into the script's entry, but for the string's id.
pub unsafe fn set_script_entry(text_id: u16) {
    unsafe {
        game_write(SCRIPT_ENTRY_RVA + 0x18, 5u8);
        game_write(SCRIPT_ENTRY_RVA + 0x19, 0x0Cu8);
        game_write(SCRIPT_ENTRY_RVA + 0x1A, text_id);
        game_write(SCRIPT_ENTRY_RVA + 0x2A, 0xBDu16);
    }
}

unsafe fn c_text<'a>(at: usize) -> &'a [u8] {
    unsafe { std::ffi::CStr::from_ptr(game_addr(at) as *const _) }.to_bytes()
}

// ---- Rust version ----

/// `no_audio` is where the sound's switch is.
pub unsafe fn check_at_start(outside: &mut dyn Outside, no_audio: *mut u32) -> Result<(), Refused> {
    unsafe {
        let record = game_addr(RECORD_RVA) as *const u8;
        if game_read::<u32>(INSTALLING_RVA) == 1 || outside.file_exists(record)? {
            return Ok(());
        }
        no_audio.write_unaligned(1);
        if outside.file_exists(game_addr(DATA_FILE_RVA) as *const u8)? {
            return Ok(());
        }
        let path = game_addr(SCRATCH_PATH_RVA) as *mut u8;
        let put = |at: usize, text: &[u8]| {
            std::ptr::copy(text.as_ptr(), path.add(at), text.len());
            path.add(at + text.len()).write(0);
        };

        outside.read_registry();
        if game_read::<u8>(STATUS_RVA) == 1 && game_read::<u8>(SOURCE_RVA) != 0 {
            // Whether the source could be put there is not asked: the program's name
            // goes after whatever text the place holds.
            outside.normalise_path(game_addr(SOURCE_RVA) as *const u8, path)?;
            let held = std::slice::from_raw_parts(path, PATH_ROOM);
            let Some(len) = held
                .iter()
                .position(|&c| c == 0)
                .filter(|len| len + 8 < PATH_ROOM)
            else {
                return Err(BadInput("the scratch path holds no text to go on from").into());
            };
            put(len, c_text(PROGRAM_RVA));
            if outside.file_exists(path)? {
                // "patch6: ", then the version from the blank before its `(`.
                let version = c_text(VERSION_RVA);
                let from = (1..version.len())
                    .find(|&n| version[n] == b'(')
                    .ok_or(BadInput("a version text without a ("))?
                    - 1;
                let text = [b"patch6: ", &version[from..], b"\n\0"].concat();
                std::ptr::copy(
                    text.as_ptr(),
                    game_addr(PATCH_TEXT_RVA) as *mut u8,
                    text.len(),
                );
                set_script_entry(0x1006);
            }
        }

        for letter in b'a'..=b'z' {
            if outside.drive_type(letter) != 5 {
                continue;
            }
            put(0, &[letter, b':', b'\\']);
            put(3, c_text(PROGRAM_RVA));
            if !outside.file_exists(path)? {
                continue;
            }
            put(3, c_text(DATA_FILE_RVA));
            if !outside.file_exists(path)? {
                continue;
            }
            path.add(3).write(0);
            if !outside.set_directory(path) {
                continue;
            }
            game_write(RUNNING_FROM_CD_RVA, 1u32);
            return Ok(());
        }
        outside.log(c"Update cannot find RC CD-ROM".as_ptr().cast());
        Err(FatalError {
            code: 0x9000_2103,
            message: "Cannot find the RC CD-ROM\nPlease insert the RC CD-ROM into your CD-ROM drive and try again",
        }
        .into())
    }
}

fn regions() -> Vec<Region> {
    vec![
        at(RUNNING_FROM_CD_RVA),
        (game_addr(SCRATCH_PATH_RVA) as u32, PATH_ROOM),
        (game_addr(PATCH_TEXT_RVA) as u32, 0x40),
        (game_addr(SCRIPT_ENTRY_RVA + 0x18) as u32, 0x14),
        (game_addr(STATUS_RVA) as u32, 0x100),
    ]
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { check_at_start(&mut Game, game_addr(NO_AUDIO_RVA) as *mut u32) }?;
    Ok(Left::nothing())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let mut regions = regions();
    regions.push(at(NO_AUDIO_RVA));
    state_entry(r, &rust)
}
