//! `menu::first_page_stage` (0x4A2440): the first stage on the last page (`dword_7556B8`)
//! for `dword_7556B4` stages, ten a page: 1 up to ten, else (n / 10) * 10 + 1, less ten when
//! n is a multiple of ten; n 0 there: the fatal error 0x80701010 (cannot happen past the
//! first test).
//!
//! In:  nothing
//! Out: unchanged up to ten; else eax the first stage, edx n % 10, ebp 10
//!
//! Callers: the event pages' code. Self-test: the original and the Rust version, counts
//! by lot.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.
use crate::framework::{game_read, game_write, rva};
use rc2k_core::menu::event::LastPage;

const COUNT_RVA: usize = rva(0x7556B4); // dword_7556B4
const FIRST_RVA: usize = rva(0x7556B8); // dword_7556B8
/// The first stage of the last page (stored in `dword_7556B8`, 1 for a single page), and
/// what the original leaves in the registers: for more than one page, eax that first
/// stage, edx the stages on the last page (count % 10), ebp 10.
pub unsafe fn first_page_stage() -> Option<LastPage> {
    let n: u32 = unsafe { game_read(COUNT_RVA) };
    // Converted: `rc2k_core::menu::event::last_page` (its division cannot meet 0).
    let Some(last) = rc2k_core::menu::event::last_page(n) else {
        unsafe { game_write(FIRST_RVA, 1u32) };
        return None;
    };
    unsafe { game_write(FIRST_RVA, last.first) };
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
