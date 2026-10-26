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

use crate::errors::Refused;
use crate::framework::Regs;
use crate::install::installer::{
    GAME_DIR, LOCALE, MEDIA_MODE, Part, TEXT, WRITE_FILE, a, at, g, gb, wb,
};
use crate::race::viewer::regs_entry;

const PATH: usize = 0x5295C0; // "var\ral.zog"

pub fn write_record(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    let start = a(TEXT);
    // Converted: `rc2k_core::install::config::record_text` (the header and keys as its
    // constants).
    let text_at = |x: usize| unsafe { std::ffi::CStr::from_ptr(a(x) as *const _) }.to_bytes();
    let locale = (gb(LOCALE) != 0).then(|| text_at(LOCALE));
    let text = rc2k_core::install::config::record_text(text_at(GAME_DIR), locale, g(MEDIA_MODE));
    for (n, &b) in text.iter().enumerate() {
        wb(start + n as u32, b);
    }
    r.ecx = text.len() as u32;
    r.eax = a(PATH);
    r.ebx = start;
    part(at(WRITE_FILE), &mut r)?;
    Ok(Regs { ..*entry })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &|r| {
        write_record(r, &mut crate::install::installer::game_part)
    })
}
