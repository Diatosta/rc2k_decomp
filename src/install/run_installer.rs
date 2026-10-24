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
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::errors::Refused;
use crate::install::installer::{
    BASE_B, FIRST_DRIVE, FROM_COMMAND, GAME_DIR, INSTALLED, INSTALLING, SAME_VERSION, SOURCE_SIZE,
    TYPED, a, append_byte, append_string_by_id, copy_picture, draw_string, fatal, g, gb,
    load_language, lock, normalise, p, pb, rb, set_cd, show, sleep, unlock, wb,
};
use crate::session::MACHINE;

const BASE_A: usize = 0x5189A8; // g_baseDirectory
const SOURCE_VAR: usize = 0x51BB8D; // g_installSource
const RECORD_FLAG: usize = 0x51D72C; // dword_51D72C
const UPDATING: usize = 0x51B9CC; // dword_51B9CC
const FROM_CD: usize = 0x529DB0; // g_runningFromCd
const UPDATE_PAGES: usize = 0x51D5D8; // off_51D5D8
const NORMAL_PAGES: usize = 0x51D594; // off_51D594
const FONT_BE: usize = 0x51E904; // stru_51E904
const LANGUAGES_PATH: usize = 0x51C36E; // "lang.cfg"
const RECORD_PATH: usize = 0x5295C0; // "var\ral.zog"

pub fn run_installer() -> Result<(), Refused> {
    use crate::install as i;
    pb(BASE_A, 0);
    unsafe { i::read_languages::read_languages(a(LANGUAGES_PATH) as *const u8) }?;
    unsafe { crate::gfx::build_fonts::build_fonts() }?;
    i::open_installer::open_installer()?;
    // From the command line: the install, and on past the pages (`jmp loc_412EDC`).
    let from_command = g(FROM_COMMAND) == 1;
    if from_command {
        i::install_from_command::install_from_command()?;
    } else {
        if !normalise(a(BASE_B), a(GAME_DIR))? {
            return Err(fatal(0x80AF_8101));
        }
        i::list_drives::list_drives();
        let cursor = append_string_by_id(0x9F, a(TYPED));
        let cursor = append_string_by_id(0xA0, cursor);
        append_byte(cursor, 0);
        if MACHINE.keyboard_type.get() == 2 {
            wb(a(TYPED), g(FIRST_DRIVE) as u8);
        }
        unsafe { i::read_registry::read_registry_here() };
        if gb(SOURCE_VAR) != 0 {
            let n = g(SOURCE_SIZE);
            for k in 0..n {
                wb(a(TYPED) + k, rb(a(SOURCE_VAR) + k));
            }
        }
        i::check_installed_version::check_installed_version()?;
        // The version-match branch changes into the installed folder.
        if g(SAME_VERSION) == 1 {
            p(RECORD_FLAG, 1);
            set_cd(a(INSTALLED));
            unsafe { i::read_record::read_record(a(RECORD_PATH) as *const u8) }?;
            pb(BASE_A, 0);
            set_cd(a(GAME_DIR));
            load_language()?;
        }
        let pages = if g(UPDATING) == 1 {
            UPDATE_PAGES
        } else {
            NORMAL_PAGES
        };
        i::run_pages::run_pages(a(pages))?;
    }
    if g(INSTALLING) == 1 && g(FROM_CD) == 1 && lock() {
        copy_picture()?;
        draw_string(0xBE, a(FONT_BE), 0x1A9, 0x113)?;
        unlock();
        show();
        sleep(2000);
    }
    i::close_installer::close_installer()?;
    crate::gfx::free_fonts::free_fonts_in_game()?;
    Ok(())
}
