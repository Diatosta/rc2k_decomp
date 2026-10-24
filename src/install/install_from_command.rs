//! `install::install_from_command` (0x412F34): the install driven by the `-i` command line
//! (two paths already in `byte_95E6F8`/`byte_95E7F8` and two digits in `dword_51BA08` /
//! `dword_51BA0C`). The game folder is made current (`files::set_current_directory`); base
//! directory B (`unk_5295B4`) and the installed folder are normalised
//! (`install::normalise_path`); the given media mode is clamped to 0..3 into `g_mediaMode`
//! (only when above 0), and only then a given language (above 0) chooses and loads it
//! (`install::choose_language`, `strings::load_language`); then the files are installed
//! (`install::install_files`). Any failing directory change or normalise ends it early.
//!
//! In:  nothing
//! Out: every register kept (`pusha` / `popa`)
//! Caller `install::run_installer`.
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::errors::Refused;
use crate::install::installer::{
    BASE_B, GAME_DIR, GIVEN_LANGUAGE, GIVEN_MODE, INSTALLED, MEDIA_MODE, TYPED, a, g,
    load_language, logged, normalise, p, set_cd,
};

pub fn install_from_command() -> Result<(), Refused> {
    if !set_cd(a(GAME_DIR)) || !normalise(a(BASE_B), a(GAME_DIR))? {
        return Ok(());
    }
    normalise(a(INSTALLED), a(TYPED))?;
    // A given media mode (above 0) is taken, held to 3, and only then a given language
    // (above 0): a mode of 0 or less goes straight to the install.
    let mode = g(GIVEN_MODE);
    if (mode as i32) > 0 {
        p(MEDIA_MODE, (mode as i32).min(3) as u32);
        let language = g(GIVEN_LANGUAGE);
        if (language as i32) > 0 {
            unsafe { crate::install::choose_language::choose_language(language) }?;
            load_language()?;
        }
    }
    logged("install", crate::install::install_files::install_files())
}
