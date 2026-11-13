//! The install driven by the `-i` command line (two paths already in `game_dir` / `installed`
//! and two digits in `given_mode` / `given_language`): the given media mode (above 0, held to 3)
//! and then a given language (above 0) are taken, and the files installed
//! (`install::install_files`). A failing directory change or normalise ends it early. Called by
//! `install::run_installer`.

use crate::errors::Refused;
use crate::install::installer::{load_language, logged, normalise, set_cd, set_media_mode};

pub fn install_from_command(
    install: &'static crate::install::owner::Install,
    o: crate::owners::Owners<'static>,
) -> Result<(), Refused> {
    if !set_cd(&install.paths.game_dir)
        || !normalise(&install.texts.base_b, &install.paths.game_dir)?
    {
        return Ok(());
    }
    normalise(&install.paths.installed, &install.paths.typed)?;
    // A given media mode (above 0) is taken, held to 3, and only then a given language
    // (above 0): a mode of 0 or less goes straight to the install.
    let mode = install.run.given_mode.get();
    if (mode as i32) > 0 {
        set_media_mode((mode as i32).min(3) as u32);
        let language = install.run.given_language.get();
        if (language as i32) > 0 {
            crate::install::choose_language::choose_language(install, language)?;
            load_language(o)?;
        }
    }
    logged(
        "install",
        crate::install::install_files::install_files(install, o),
    )
}
