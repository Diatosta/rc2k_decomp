//! `install::write_record` (0x411186): the installation record `var\ral.zog` written: a
//! header, `cdrom=<folder installed into>` (`game_dir`), `language=<...>` only if
//! `g_localeDirectory` is set, and `installation=<g_mediaMode>`, each line ended CR LF. The
//! text is built in the shared buffer (`text`) and written (`files::write_file`).
//!
//! `install::read_record` reads it back. Caller `install::install_files`. The write through
//! `part` (`files::write_file`).

use crate::errors::Refused;
use crate::install::installer::media_mode;

/// The record written; whether the write worked (the original passes `write_file`'s carry
/// on, and `install_files` reads it).
pub fn write_record(
    install: &'static crate::install::owner::Install,
    strings: &'static crate::strings::owner::Strings,
) -> Result<bool, Refused> {
    let start = crate::place::BytePlace::from(&strings.text.buffer.text);
    // Converted: `rc2k_core::install::config::record_text` (the header and keys as its
    // constants).
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
