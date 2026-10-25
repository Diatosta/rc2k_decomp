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
use crate::install::vars::INSTALLER_NAMES;
use crate::install::vars::INSTALLER_PATHS;
use crate::install::vars::INSTALLER_RUN;

pub fn open_installed() -> Result<(), Refused> {
    if normalise(
        INSTALLER_NAMES.source.addr() as u32,
        INSTALLER_PATHS.installed.addr() as u32,
    )? && set_cd(INSTALLER_PATHS.installed.addr() as u32)
    {
        INSTALLER_RUN.installing.set(1);
    }
    Ok(())
}
