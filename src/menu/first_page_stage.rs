//! `menu::first_page_stage` (0x4A2440): the first stage on the last page (`dword_7556B8`)
//! for `dword_7556B4` stages, ten a page: 1 up to ten, else (n / 10) * 10 + 1, less ten when
//! n is a multiple of ten; n 0 there: the fatal error 0x80701010 (cannot happen past the
//! first test).
//!
//! In:  nothing
//! Out: unchanged up to ten; else eax the first stage, edx n % 10, ebp 10
//!
//! Callers: the event pages' code.
use crate::menu::champ_store::CHAMP_VIEW;

/// The first stage of the last page, stored in `dword_7556B8` (1 for a single page). (What
/// the original also leaves in eax, edx and ebp, nothing reads.)
pub fn first_page_stage() {
    let n: u32 = CHAMP_VIEW.count.get();
    // Converted: `rc2k_core::menu::event::last_page` (its division cannot meet 0).
    let first = rc2k_core::menu::event::last_page(n).map_or(1, |last| last.first);
    CHAMP_VIEW.first_row.set(first);
}
