//! `install::run_installer` (0x412DE0): the installer's branch of the main loop. The base
//! directory is cleared, the languages read (`install::read_languages`), the fonts built
//! (`gfx::build_fonts`) and the installer opened (`install::open_installer`). With the `-i`
//! paths given (`dword_51B9D0`), `install::install_from_command` does it all; otherwise base
//! directory B is normalised (failing: fatal error 0x80AF8101), the drives listed
//! (`install::list_drives`), a default folder built in `byte_95EBF8` (strings 0x9F, 0xA0, the
//! first drive letter when `g_keyboardType` is 2), the registry read, the recorded source
//! copied over it, and the installed version checked (`install::check_installed_version`); if
//! it is this build, its record is read and language loaded. Then the pages run
//! (`install::run_pages` with `off_51D5D8` for an update, else `off_51D594`); if installing
//! from a CD a "please wait" frame is shown. Finally the installer is closed
//! (`install::close_installer`) and the fonts freed (`gfx::free_fonts`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` / `popa`)
//! Caller `system::main_loop` (the Rust version calls this directly). Its calls through
//! `part`; the default-folder strings built directly. Ungated. Shadow mode runs the original
//! alone. Self-test: a copy whose calls go to stand-ins, the branch globals scripted, against
//! the Rust version; the given-paths path, the normal path, an already-installed build, an
//! update, installing-from-CD, and the normalise fatal.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::install::installer::{
    BASE_B, FIRST_DRIVE, FROM_COMMAND, GAME_DIR, INSTALLING, Part, SAME_VERSION, SOURCE_SIZE,
    TYPED, a, append_byte, append_string_by_id, at, fatal, g, gb, p, pb, rb, wb,
};

const BASE_A: usize = 0x5189A8; // g_baseDirectory
const READ_LANGUAGES: usize = 0x413908;
const BUILD_FONTS: usize = 0x415563;
const OPEN_INSTALLER: usize = 0x414363;
const INSTALL_FROM_COMMAND: usize = 0x412F34;
const NORMALISE: usize = 0x4139EC;
const LIST_DRIVES: usize = 0x413C27;
const READ_REGISTRY: usize = 0x413D14;
const CHECK_VERSION: usize = 0x413B0B;
const SET_CD: usize = 0x403105;
const READ_RECORD: usize = 0x4110C9;
const LOAD_LANGUAGE: usize = 0x40331F;
const RUN_PAGES: usize = 0x4144AA;
const LOCK: usize = 0x4058AD;
const COPY_PICTURE: usize = 0x4060E3;
const DRAW_STRING: usize = 0x40234A;
const UNLOCK: usize = 0x405949;
const SHOW: usize = 0x4057D1;
const SLEEP: usize = 0x41100C;
const CLOSE_INSTALLER: usize = 0x414490;
const FREE_FONTS: usize = 0x415C55;
const KEYBOARD_TYPE: usize = 0x518404; // g_keyboardType
const SOURCE_VAR: usize = 0x51BB8D; // g_installSource
const RECORD_FLAG: usize = 0x51D72C; // dword_51D72C
const UPDATING: usize = 0x51B9CC; // dword_51B9CC
const FROM_CD: usize = 0x529DB0; // g_runningFromCd
const UPDATE_PAGES: usize = 0x51D5D8; // off_51D5D8
const NORMAL_PAGES: usize = 0x51D594; // off_51D594
const FONT_BE: usize = 0x51E904; // stru_51E904

pub fn run_installer(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    pb(BASE_A, 0);
    part(at(READ_LANGUAGES), &mut r)?;
    part(at(BUILD_FONTS), &mut r)?;
    part(at(OPEN_INSTALLER), &mut r)?;
    // From the command line: the install, and on past the pages (`jmp loc_412EDC`).
    let from_command = g(FROM_COMMAND) == 1;
    if from_command {
        part(at(INSTALL_FROM_COMMAND), &mut r)?;
    } else {
        r.esi = a(BASE_B);
        r.edx = a(GAME_DIR);
        part(at(NORMALISE), &mut r)?;
        if r.carry() {
            return Err(fatal(0x80AF_8101));
        }
        part(at(LIST_DRIVES), &mut r)?;
        r.ebx = a(TYPED);
        r.eax = 0x9F;
        append_string_by_id(&mut r);
        r.eax = 0xA0;
        append_string_by_id(&mut r);
        append_byte(&mut r, 0);
        if gb(KEYBOARD_TYPE) == 2 {
            r.eax = g(FIRST_DRIVE);
            wb(a(TYPED), r.eax as u8);
        }
        part(at(READ_REGISTRY), &mut r)?;
        if gb(SOURCE_VAR) != 0 {
            let n = g(SOURCE_SIZE);
            for k in 0..n {
                wb(a(TYPED) + k, rb(a(SOURCE_VAR) + k));
            }
        }
        part(at(CHECK_VERSION), &mut r)?;
        if g(SAME_VERSION) == 1 {
            p(RECORD_FLAG, 1);
            r.edx = a(GAME_DIR_INSTALLED());
            part(at(SET_CD), &mut r)?;
            part(at(READ_RECORD), &mut r)?;
            pb(BASE_A, 0);
            r.edx = a(GAME_DIR);
            part(at(SET_CD), &mut r)?;
            part(at(LOAD_LANGUAGE), &mut r)?;
        }
    }
    if !from_command {
        let pages = if g(UPDATING) == 1 {
            UPDATE_PAGES
        } else {
            NORMAL_PAGES
        };
        r.esi = a(pages);
        part(at(RUN_PAGES), &mut r)?;
    }
    if g(INSTALLING) == 1 && g(FROM_CD) == 1 {
        part(at(LOCK), &mut r)?;
        if !r.carry() {
            part(at(COPY_PICTURE), &mut r)?;
            r.eax = 0xBE;
            r.ebx = a(FONT_BE);
            r.ecx = 0x1A9;
            r.edx = 0x113;
            part(at(DRAW_STRING), &mut r)?;
            part(at(UNLOCK), &mut r)?;
            part(at(SHOW), &mut r)?;
            r.eax = 2000;
            part(at(SLEEP), &mut r)?;
        }
    }
    part(at(CLOSE_INSTALLER), &mut r)?;
    part(at(FREE_FONTS), &mut r)?;
    Ok(Regs { ..*entry })
}

/// byte_95E7F8 (the installed folder) — the version-match branch changes into it.
#[allow(non_snake_case)]
fn GAME_DIR_INSTALLED() -> usize {
    0x95E7F8
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    crate::race::viewer::regs_entry(r, &|r| {
        run_installer(r, &mut crate::install::installer::game_part)
    })
}
