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

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::install::installer::{
    GAME_DIR, LOCALE, MEDIA_MODE, Part, TEXT, WRITE_FILE, a, at, both, g, gb, quiet, region, wb,
};
use crate::race::viewer::{Saved, dispatch_parts, regs_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("install::write_record", rva(0x411186), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

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

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_parts(&HOOK, r, call_original, &|r| {
        write_record(r, &mut crate::install::installer::game_part)
    });
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &|r| {
        write_record(r, &mut crate::install::installer::game_part)
    })
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; lea ebx, text
    let copy = match crate::install::installer::copy(
        0x411186,
        0xB3,
        &[0x60, 0x8D, 0x1D, 0x30, 0xE4, 0x93, 0x00],
        &[
            (0xE, 0x401F7D),
            (0x19, 0x401F7D),
            (0x24, 0x401F7D),
            (0x2F, 0x401F7D),
            (0x3A, 0x401F7D),
            (0x4E, 0x401F7D),
            (0x59, 0x401F7D),
            (0x64, 0x401F7D),
            (0x6F, 0x401F7D),
            (0x7A, 0x401F7D),
            (0x85, 0x401F7D),
            (0x91, 0x40204A),
            (0x9C, 0x401F7D),
            (0xAC, 0x402BFB),
        ],
        &[0x402BFB],
        &[],
        &[],
        &[],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    quiet();
    crate::install::installer::set_text_of(Some(crate::install::installer::path_text));
    let regions = [
        region(TEXT, 0x100),
        region(GAME_DIR, 0x20),
        region(LOCALE, 2),
        region(MEDIA_MODE, 4),
    ];
    let saved = Saved::new(&regions);
    for (lang, mode) in [(0u8, 0u32), (b'f', 2), (0, 3)] {
        let prepare = || {
            crate::install::installer::fill(a(TEXT), 0x100, 0x22);
            for (k, &b) in b"c:\\games\\rally\\\0".iter().enumerate() {
                wb(a(GAME_DIR) + k as u32, b);
            }
            wb(a(LOCALE), lang);
            if lang != 0 {
                wb(a(LOCALE) + 1, 0);
            }
            crate::install::installer::p(MEDIA_MODE, mode);
        };
        both(
            &mut t,
            &format!("language {lang:#x}, mode {mode}"),
            &copy,
            Regs::SENTINEL,
            &regions,
            0,
            &[],
            &prepare,
            &|r, part, _| write_record(r, part),
        );
    }
    crate::install::installer::set_text_of(None);
    drop(saved);
    t.finish()
}
