//! `menu::show_class_rally_results` (0x4A3317): the rally results of a class: `dword_7558F4` the class (ebx); the first player's entry of class ebx in standings
//! table A (`menu::first_player_a`), its figure held to 0x51 (event mode 0x13) or 0x33 into `dword_7556B4`;
//! `menu::first_page_stage`, `menu::stage_range_caption`; then the page
//! (`menu::class_rally_results_loop`).
//!
//! In:  ebx the class
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `sub_4A32A5`. The steps are their Rust versions; the page through `part` (1), in
//! the game its Rust version (`screen_loop::run_screen`). Ungated (the page waits for the
//! player). Refuses what the steps refuse, before anything; raises the search's fatal
//! error as the original. Self-test: a copy whose page call goes to a stand-in, against
//! the Rust version, on a standings table of the test's.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, game_write, rva};
use crate::menu::standings::*;
use rc2k_core::menu::results_pages::{Page, held_figure};

const CLASS_RVA: usize = rva(0x7558F4); // dword_7558F4
const COUNT_RVA: usize = rva(0x7556B4); // dword_7556B4
const EVENT_MODE_RVA: usize = rva(0x7556D0); // g_eventMode

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn show_class_rally_results(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        g.eax = game_read(rva(0x7559AC)); // seconds
        game_write(CLASS_RVA, g.ebx);
        g.edi = game_read(TABLE_A_RVA);
        let found = crate::menu::first_player_a::first_player_a(&g)?;
        // Converted: `rc2k_core::menu::results_pages::held_figure`.
        let a8 = game_read::<u32>(EVENT_MODE_RVA) == rc2k_core::menu::results_pages::A8;
        let count = held_figure(Page::ClassRally { a8 }, found.edx);
        game_write(COUNT_RVA, count);
        let g = crate::menu::first_page_stage::first_page_stage(&found)?;
        crate::menu::stage_range_caption::stage_range_caption();
        let mut r = g;
        crate::menu::show_class_rally_results::game_part(1, &mut r)?;
    }
    Ok(*entry)
}

/// In the game: the page's Rust version.
pub fn game_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        crate::menu::screen_loop::run_screen(&crate::menu::class_rally_results_loop::SCREEN, r)
    };
    Ok(())
}
