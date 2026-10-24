//! `install::language_page` (0x414199): the language page. At the start (ebx 0) with only
//! one language (`g_languageCount` <= 1) or this version already installed (`dword_51B9C8`
//! 1), the list's `[edi]` gets 0x40 (the page is skipped). At the end (ebx negative) with a
//! chosen answer (`byte_51D415` != 0) the language is chosen (`install::choose_language`)
//! and loaded (`strings::load_language`). Then the items past the languages
//! (`g_languageCount` of them, up to 9) get 0x40 (hidden).
//!
//! In:  ebx the state, edi the list
//! Out: eax and what the callees leave; the flags of the hide loop (no caller reads them)
//! A page action. Caller `install::run_page`.
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::errors::Refused;
use crate::install::installer::{
    LANGUAGE_COUNT, LIST_ANSWER, SAME_VERSION, g, gb, load_language, rb, wb,
};
use rc2k_core::install::pages;

/// `state` the list action's (ebx), `list` the page's.
pub fn language_page(state: u32, list: u32) -> Result<(), Refused> {
    if state == 0 {
        // The same version, or at most one language (unsigned): the page itself hidden
        // (`or byte ptr [edi], 40h`). Else on to the hide loop (`loc_4141B4`).
        // Converted: `rc2k_core::install::pages` (`language_page_skipped`,
        // `hidden_languages`).
        if pages::language_page_skipped(g(SAME_VERSION) == 1, g(LANGUAGE_COUNT)) {
            wb(list, rb(list) | pages::HIDDEN);
            return Ok(());
        }
    } else if (state as i32) < 0 {
        let answer = gb(LIST_ANSWER) as u32;
        if answer != 0 {
            unsafe { crate::install::choose_language::choose_language(answer) }?;
            load_language()?;
        }
    }
    // hide the items past the languages
    let (first, hidden) = pages::hidden_languages(g(LANGUAGE_COUNT));
    let mut item = first.wrapping_add(list).wrapping_add(0x18);
    for _ in 0..hidden {
        wb(item, rb(item) | pages::HIDDEN);
        item = item.wrapping_add(0x10);
    }
    Ok(())
}
