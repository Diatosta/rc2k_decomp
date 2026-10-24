//! `install::open_installed` (0x413629): the installed folder (`g_installSource` normalised
//! into `byte_95E7F8`) made the current directory and `g_installing` set; if either the
//! normalise or the change fails, nothing is set.
//!
//! In:  nothing
//! Out: every register kept (`pusha` / `popa`)
//! A list action (`off_51DD48` for the update script). Caller `install::run_page`. Its two
//! calls through `part`. Shadow mode runs the original alone (it changes directory).
//! Self-test: a copy whose two calls go to stand-ins, against the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::errors::Refused;
use crate::install::installer::{INSTALLED, INSTALLING, a, normalise, p, set_cd};

const SOURCE_PATH: usize = 0x51BB8D; // g_installSource

pub fn open_installed() -> Result<(), Refused> {
    if normalise(a(SOURCE_PATH), a(INSTALLED))? && set_cd(a(INSTALLED)) {
        p(INSTALLING, 1);
    }
    Ok(())
}
