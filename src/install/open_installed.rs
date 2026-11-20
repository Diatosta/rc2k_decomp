//! The installed folder (the install source normalised into `installed`) made the current directory
//! and the installing flag set; if the normalise or the change fails, nothing is set.

use crate::errors::Refused;
use crate::install::installer::{normalise, set_cd};

pub(super) fn open_installed(
    install: &'static crate::install::owner::Install,
) -> Result<(), Refused> {
    if let Some(installed) = normalise(&install.names.source.text())? {
        install.paths.installed.set(&installed)?;
        if set_cd(&install.paths.installed.text()) {
            install.flags.installing.set(true);
        }
    }
    Ok(())
}
