//! The installation record `var\ral.zog` written: a header, `cdrom=<folder installed into>`,
//! `language=<...>` only if the locale directory is set, and `installation=<media mode>`, each
//! line ended CR LF. `install::read_record` reads it back. Called by `install::install_files`.

use crate::errors::Refused;
use crate::install::installer::media_mode;

/// The record written; whether the write worked (`install_files` reads it).
pub fn write_record(
    install: &'static crate::install::owner::Install,
    strings: &'static crate::strings::owner::Strings,
) -> Result<bool, Refused> {
    let start = crate::place::BytePlace::from(&strings.text.buffer.text);
    let locale = (crate::files::state::directory_byte(1) != 0)
        .then(|| crate::files::state::directory_text(1, "the locale directory"))
        .transpose()?;
    let text = rc2k_core::install::config::record_text(
        &crate::place::BytePlace::from(&install.paths.game_dir).text("the game's folder")?,
        locale.as_deref(),
        media_mode(),
    );
    start.write(&text, "the install record's text")?;
    // files::write_file
    let written =
        crate::files::write_file::write_file_from(&install.texts.record_path.text()[..], &text)?;
    Ok(written.ok)
}
