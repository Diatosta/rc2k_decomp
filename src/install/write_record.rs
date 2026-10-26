//! `install::write_record` (0x411186): the installation record `var\ral.zog` written: a
//! header, `cdrom=<folder installed into>` (`byte_95E6F8`), `language=<...>` only if
//! `g_localeDirectory` is set, and `installation=<g_mediaMode>`, each line ended CR LF. The
//! text is built in the shared buffer (`text`) and written (`files::write_file`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` / `popa`)
//! `install::read_record` reads it back. Caller `install::install_files`. The write through
//! `part` (`files::write_file`).
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::errors::Refused;
use crate::install::installer::{media_mode, wb};
use crate::install::vars::INSTALLER_PATHS;
use crate::pointers::GamePtr;
use crate::strings::text_buffer::TEXT_BUFFER;

/// The record written; whether the write worked (the original leaves `write_file`'s carry
/// through its `pusha` / `popa`, and `install_files` reads it). Until 2026-10-23 the port
/// dropped it, and the install was judged by the change of folder before it.
pub fn write_record() -> Result<bool, Refused> {
    let start = TEXT_BUFFER.text.addr() as u32;
    // Converted: `rc2k_core::install::config::record_text` (the header and keys as its
    // constants).
    let text_at = |x: u32, what| crate::files::cstring(GamePtr::new(x), what);
    let locale_at = crate::pointers::game_address(crate::files::state::locale_directory());
    let locale = (GamePtr::new(locale_at).read::<u8>("the locale directory")? != 0)
        .then(|| text_at(locale_at, "the locale directory"))
        .transpose()?;
    let text = rc2k_core::install::config::record_text(
        &text_at(INSTALLER_PATHS.game_dir.addr() as u32, "the game's folder")?,
        locale.as_deref(),
        media_mode(),
    );
    for (n, &b) in text.iter().enumerate() {
        wb(start + n as u32, b);
    }
    // files::write_file
    let written = crate::files::write_file::write_file(
        crate::pointers::host_at(crate::install::installer::INSTALL_TEXTS.record_path.addr())
            as *const u8,
        crate::pointers::host(start) as *const u8,
        text.len() as u32,
    )?;
    Ok(written.ok)
}
