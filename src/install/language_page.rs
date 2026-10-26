//! `install::language_page` (0x414199): the language page. At the start (ebx 0) with only
//! one language (`g_languageCount` <= 1) or this version already installed (`dword_51B9C8`
//! 1), the list's `[edi]` gets 0x40 (the page is skipped). At the end (ebx negative) with a
//! chosen answer (`byte_51D415` != 0) the language is chosen (`install::choose_language`)
//! and loaded (`strings::load_language`). Then the items past the languages
//! (`g_languageCount` of them, up to 9) get 0x40 (hidden).
//!
//! In:  ebx the state, edi the list
//! Out: eax and what the callees leave; the flags of the hide loop (no caller reads them)
//! A page action. Caller `install::run_page`. Its two calls through `part`. Ungated
//! (loading the language reads files and compiles). Shadow mode runs the original alone.
//! Self-test: a copy whose two calls go to stand-ins, against the Rust version, every
//! state and language count.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::install::installer::{
    CHOOSE_LANGUAGE, LANGUAGE_COUNT, LIST_ANSWER, LOAD_LANGUAGE, Part, SAME_VERSION, at, g, gb,
    logic, rb, wb,
};
use crate::race::viewer::regs_entry;
use rc2k_core::install::pages;

pub fn language_page(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    if r.ebx == 0 {
        // The same version, or at most one language (unsigned): the item itself hidden
        // (`or byte ptr [edi], 40h`, its flags). Else on to the hide loop (`loc_4141B4`).
        // Converted: `rc2k_core::install::pages` (`language_page_skipped`,
        // `hidden_languages`).
        if pages::language_page_skipped(g(SAME_VERSION) == 1, g(LANGUAGE_COUNT)) {
            let v = rb(r.edi) | pages::HIDDEN;
            wb(r.edi, v);
            logic(&mut r, v as u32);
            return Ok(r);
        }
    } else if (r.ebx as i32) < 0 {
        r.eax = gb(LIST_ANSWER) as u32;
        if r.eax != 0 {
            part(at(CHOOSE_LANGUAGE), &mut r)?;
            part(at(LOAD_LANGUAGE), &mut r)?;
        }
    }
    // hide the items past the languages
    let (first, hidden) = pages::hidden_languages(g(LANGUAGE_COUNT));
    r.edx = first.wrapping_add(r.edi).wrapping_add(0x18);
    r.ecx = hidden;
    let c = r.ecx;
    logic(&mut r, c);
    if r.ecx != 0 {
        loop {
            wb(r.edx, rb(r.edx) | pages::HIDDEN);
            r.edx = r.edx.wrapping_add(0x10);
            r.ecx = r.ecx.wrapping_sub(1);
            if r.ecx == 0 {
                break;
            }
        }
        logic(&mut r, 0);
    }
    Ok(r)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &|r| {
        language_page(r, &mut crate::install::installer::game_part)
    })
}
