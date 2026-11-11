//! `install::open_installed` (0x413629): the installed folder (`g_installSource` normalised
//! into `installed`) made the current directory and `g_installing` set; if either the
//! normalise or the change fails, nothing is set.
//!
//! A list action (`step` for the update script). Caller `install::run_page`.

use crate::errors::Refused;
use crate::install::installer::{normalise, set_cd};

pub fn open_installed(install: &'static crate::install::owner::Install) -> Result<(), Refused> {
    if normalise(&install.names.source, &install.paths.installed)?
        && set_cd(&install.paths.installed)
    {
        install.run.installing.set(1);
    }
    Ok(())
}
