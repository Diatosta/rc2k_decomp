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
use rc2k_core::menu::event::LastPage;

/// The first stage of the last page (stored in `dword_7556B8`, 1 for a single page), and
/// what the original leaves in the registers: for more than one page, eax that first
/// stage, edx the stages on the last page (count % 10), ebp 10.
pub unsafe fn first_page_stage() -> Option<LastPage> {
    let n: u32 = { CHAMP_VIEW.count.get() };
    // Converted: `rc2k_core::menu::event::last_page` (its division cannot meet 0).
    let Some(last) = rc2k_core::menu::event::last_page(n) else {
        CHAMP_VIEW.first_row.set(1u32);
        return None;
    };
    CHAMP_VIEW.first_row.set(last.first);
    Some(last)
}

/// The carried values as `first_page_stage` leaves them (ebp, which it also sets, is
/// nothing a screen reads).
pub fn carry_on(last: Option<LastPage>, c: &mut crate::menu::screen_loop::Carried) {
    if let Some(last) = last {
        c.clock = last.rest;
        c.arg = last.first;
    }
}
