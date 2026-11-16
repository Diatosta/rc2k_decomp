//! The installed folder (the install source normalised into `installed`) made the current
//! directory and the installing flag set; if the normalise or the change fails, nothing is set.
//! Called by `install::run_pages`.

use crate::errors::Refused;
use crate::install::installer::{normalise, set_cd};

pub fn open_installed(install: &'static crate::install::owner::Install) -> Result<(), Refused> {
    if normalise(&install.names.source, &install.paths.installed)?
        && set_cd(&install.paths.installed)
    {
        install.flags.installing.set(true);
    }
    Ok(())
}
