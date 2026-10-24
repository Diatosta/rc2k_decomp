//! `menu::show_overall_rally_results` (0x4A3BA2): the overall rally results: the first player's entry of any group in standings
//! table A (`menu::first_player_a`), its figure held to 0x51 when above 0xAB (the original's: section 16) into `dword_7556B4`;
//! `menu::first_page_stage`, `menu::stage_range_caption`; then the page
//! (`menu::overall_rally_results_loop`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: `sub_4A0E60` (twice), `sub_4A94EC`. The steps are their Rust versions; the page through `part` (1), in
//! the game its Rust version (`screen_loop::run_screen`). Ungated (the page waits for the
//! player). Refuses what the steps refuse, before anything; raises the search's fatal
//! error as the original. Self-test: a copy whose page call goes to a stand-in, against
//! the Rust version, on a standings table of the test's.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{game_read, game_write, rva};
use crate::menu::screen_loop::{Carried, run_screen};
use crate::menu::standings::*;
use rc2k_core::menu::results_pages::{Page, held_figure};

// dword_7558F4
const COUNT_RVA: usize = rva(0x7556B4); // dword_7556B4
// g_eventMode

/// The Rust version (see the header). `c` the carried values (the registers the page's
/// screen is run with).
pub unsafe fn show_overall_rally_results(c: &Carried) -> Result<(), Refused> {
    unsafe {
        let mut c = Carried {
            palette: ANY_GROUP,
            ..*c
        };
        let found = crate::menu::first_player_a::first_player_a(game_read(TABLE_A_RVA), ANY_GROUP)?;
        found.carry_on(&mut c);
        // Converted: `rc2k_core::menu::results_pages::held_figure`.
        let count = held_figure(Page::OverallRally, found.figure);
        game_write(COUNT_RVA, count);
        crate::menu::first_page_stage::carry_on(
            crate::menu::first_page_stage::first_page_stage(),
            &mut c,
        );
        crate::menu::stage_range_caption::stage_range_caption();
        run_screen(&crate::menu::overall_rally_results_loop::SCREEN, &c)?;
    }
    Ok(())
}
