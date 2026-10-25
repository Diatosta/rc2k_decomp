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
//! One caller, `start`, which calls it directly (2026-10-23). This is not the copy protection's CD check (that one is in the
//! main loop and `nop`ped out in this build).
//! Refuses a scratch path that is not a text when the source cannot be put there.

use crate::errors::{BadInput, FatalError, Refused};
use crate::install::outside::Outside;
use crate::install::runs_from_source::scratch_path;
use crate::install::vars::INSTALLER_NAMES;
use crate::install::vars::INSTALLER_PAGES;
use crate::install::vars::INSTALLER_RUN;
use crate::system::start_texts::START_TEXTS;

const PATH_ROOM: usize = 0x100;

// ---- game data ----

/// What both checks write into the script's entry, but for the string's id.
pub fn set_script_entry(text_id: u16) {
    // The entry of the installer's script (off_51D728): two bytes, a string's id, and
    // 0xBD (`word_51D752`) = the installer runs.
    INSTALLER_PAGES.entry_kind.set(5);
    INSTALLER_PAGES.entry_marks.set(0x0C);
    INSTALLER_PAGES.entry_text.set(text_id);
    INSTALLER_PAGES.script_set_up.set(0xBD);
}

/// "ral.exe" (in ".\ral.exe").
unsafe fn program<'a>() -> &'a [u8] {
    unsafe { std::ffi::CStr::from_ptr(INSTALLER_NAMES.program.addr() as *const _) }.to_bytes()
}

// ---- Rust version ----

/// `no_audio` is where the sound's switch is.
pub unsafe fn check_at_start(outside: &mut dyn Outside, no_audio: *mut u32) -> Result<(), Refused> {
    unsafe {
        let record = crate::install::installer::INSTALL_TEXTS.record_path.addr() as *const u8;
        if INSTALLER_RUN.installing.get() == 1 || outside.file_exists(record)? {
            return Ok(());
        }
        no_audio.write_unaligned(1);
        if outside.file_exists(crate::gfx::texts::address(&crate::gfx::texts::JADE) as *const u8)? {
            return Ok(());
        }
        let path = scratch_path();
        let put = |at: usize, text: &[u8]| {
            std::ptr::copy(text.as_ptr(), path.add(at), text.len());
            path.add(at + text.len()).write(0);
        };

        outside.read_registry();
        if INSTALLER_NAMES.status.get() == 1 && INSTALLER_NAMES.source.get(0) != 0 {
            // Whether the source could be put there is not asked: the program's name
            // goes after whatever text the place holds.
            outside.normalise_path(INSTALLER_NAMES.source.addr() as *const u8, path)?;
            let held = std::slice::from_raw_parts(path, PATH_ROOM);
            let Some(len) = held
                .iter()
                .position(|&c| c == 0)
                .filter(|len| len + 8 < PATH_ROOM)
            else {
                return Err(BadInput("the scratch path holds no text to go on from").into());
            };
            put(len, program());
            if outside.file_exists(path)? {
                // "patch6: ", then the version from the blank before its `(`.
                let version =
                    std::ffi::CStr::from_ptr(START_TEXTS.version.addr() as *const _).to_bytes();
                let from = (1..version.len())
                    .find(|&n| version[n] == b'(')
                    .ok_or(BadInput("a version text without a ("))?
                    - 1;
                let text = [b"patch6: ", &version[from..], b"\n\0"].concat();
                std::ptr::copy(
                    text.as_ptr(),
                    crate::screens::text_buffers_store::TEXT_BUFFERS
                        .number_text
                        .addr() as *mut u8,
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
            put(3, program());
            if !outside.file_exists(path)? {
                continue;
            }
            put(
                3,
                &crate::gfx::texts::JADE[..crate::gfx::texts::JADE.len() - 1],
            );
            if !outside.file_exists(path)? {
                continue;
            }
            path.add(3).write(0);
            if !outside.set_directory(path) {
                continue;
            }
            crate::install::store::INSTALL.running_from_cd.set(1);
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
