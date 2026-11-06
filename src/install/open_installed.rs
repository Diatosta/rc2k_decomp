//! `install::open_installed` (0x413629): the installed folder (`g_installSource` normalised
//! into `byte_95E7F8`) made the current directory and `g_installing` set; if either the
//! normalise or the change fails, nothing is set.
//!
//! In:  nothing
//! Out: every register kept (`pusha` / `popa`)
//! A list action (`off_51DD48` for the update script). Caller `install::run_page`.
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::errors::Refused;
use crate::install::installer::{normalise, set_cd};

pub fn open_installed(g: &'static crate::install::owner::Install) -> Result<(), Refused> {
    if normalise(&g.names.source, &g.paths.installed)? && set_cd(&g.paths.installed) {
        g.run.installing.set(1);
    }
    Ok(())
}
