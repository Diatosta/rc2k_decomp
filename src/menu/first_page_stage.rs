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
use crate::errors::Refused;
use crate::framework::{Regs, game_read, game_write, rva};

const COUNT_RVA: usize = rva(0x7556B4); // dword_7556B4
const FIRST_RVA: usize = rva(0x7556B8); // dword_7556B8
/// The Rust version (see the header): the registers it leaves.
pub unsafe fn first_page_stage(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    let n: u32 = unsafe { game_read(COUNT_RVA) };
    // Converted: `rc2k_core::menu::event::last_page` (its division cannot meet 0).
    let Some(last) = rc2k_core::menu::event::last_page(n) else {
        unsafe { game_write(FIRST_RVA, 1u32) };
        return Ok(g);
    };
    g.edx = last.rest;
    g.eax = last.first;
    g.ebp = 10;
    unsafe { game_write(FIRST_RVA, g.eax) };
    Ok(g)
}
