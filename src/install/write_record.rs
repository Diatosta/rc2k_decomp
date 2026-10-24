//! `install::write_record` (0x411186): the installation record `var\ral.zog` written: a
//! header, `cdrom=<folder installed into>` (`byte_95E6F8`), `language=<...>` only if
//! `g_localeDirectory` is set, and `installation=<g_mediaMode>`, each line ended CR LF. The
//! text is built in the shared buffer (`text`) and written (`files::write_file`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` / `popa`)
//! `install::read_record` reads it back. Caller `install::install_files`. The write through
//! `part` (`files::write_file`). Shadow mode runs the original alone (one write). Self-test:
//! a copy whose one write goes to a stand-in (the bytes noted), against the Rust version,
//! with and without a language.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::errors::Refused;
use crate::install::installer::{GAME_DIR, LOCALE, MEDIA_MODE, TEXT, a, g, gb, wb};

const PATH: usize = 0x5295C0; // "var\ral.zog"

/// The record written; whether the write worked (the original leaves `write_file`'s carry
/// through its `pusha` / `popa`, and `install_files` reads it). Until 2026-10-23 the port
/// dropped it, and the install was judged by the change of folder before it.
pub fn write_record() -> Result<bool, Refused> {
    let start = a(TEXT);
    // Converted: `rc2k_core::install::config::record_text` (the header and keys as its
    // constants).
    let text_at = |x: usize| unsafe { std::ffi::CStr::from_ptr(a(x) as *const _) }.to_bytes();
    let locale = (gb(LOCALE) != 0).then(|| text_at(LOCALE));
    let text = rc2k_core::install::config::record_text(text_at(GAME_DIR), locale, g(MEDIA_MODE));
    for (n, &b) in text.iter().enumerate() {
        wb(start + n as u32, b);
    }
    // files::write_file
    let written = unsafe {
        crate::files::write_file::write_file(
            a(PATH) as *const u8,
            start as *const u8,
            text.len() as u32,
        )
    }?;
    Ok(written.ok)
}
