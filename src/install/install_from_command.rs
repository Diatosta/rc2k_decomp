//! The install driven by the `-i` command line (two paths already in `game_dir` / `installed` and
//! two digits in `flags.given_mode` / `flags.given_language`): with the first digit above 0 a given
//! language (above 0) is taken, and the files installed (`install::install_files`, always the full
//! installation, whatever kind the first digit named). A failing directory change or normalise ends
//! it early.

use crate::errors::Refused;
use crate::install::installer::{load_language, logged, normalise, set_cd};

pub fn install_from_command(
    install: &'static crate::install::owner::Install,
    o: crate::owners::Owners<'static>,
) -> Result<(), Refused> {
    if !set_cd(&install.paths.game_dir)
        || !normalise(&crate::files::state::SECOND_BASE, &install.paths.game_dir)?
    {
        return Ok(());
    }
    normalise(&install.paths.installed, &install.paths.typed)?;
    // Only with the first digit above 0 is a given language taken.
    if install.flags.given_mode.get() > 0 {
        let language = install.flags.given_language.get();
        if language > 0 {
            crate::install::choose_language::choose_language(install, language as u32)?;
            load_language(o)?;
        }
    }
    logged(
        "install",
        crate::install::install_files::install_files(install, o),
    )?;
    Ok(())
}
