//! `menu::show_championship_standings` (0x4A5A47): the championship standings: the first player's entry of a group in standings
//! table B (`menu::first_player_unplaced`), its figure held to 0xAB into `dword_7556B4`;
//! `menu::first_page_stage`, `menu::stage_range_caption`; then the page
//! (`menu::championship_standings_loop`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: `sub_4A0E60`, `sub_4A9583`. The steps are their Rust versions; the page through `part` (1), in
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

// dword_7558F4
const COUNT_RVA: usize = rva(0x7556B4); // dword_7556B4
// g_eventMode

/// A call by place: 1 the page.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn show_championship_standings(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        g.edi = game_read(TABLE_B_RVA);
        g.ebx = ANY_GROUP;
        let found = crate::menu::first_player_unplaced::first_player_unplaced(&g)?;
        let count = found.edx;
        // Converted: `rc2k_core::menu::results_pages::held_figure`.
        let count = held_figure(Page::ChampionshipStandings, count);
        game_write(COUNT_RVA, count);
        let g = crate::menu::first_page_stage::first_page_stage(&found)?;
        let mut r = crate::menu::stage_range_caption::stage_range_caption(&g)?;
        part(1, &mut r)?;
    }
    Ok(*entry)
}

/// In the game: the page's Rust version.
pub fn game_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        crate::menu::screen_loop::run_screen(&crate::menu::championship_standings_loop::SCREEN, r)
    };
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { show_championship_standings(r, &mut game_part) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
