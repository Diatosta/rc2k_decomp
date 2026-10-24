//! `menu::best_overall_scores` (0x4A91BF): the overall new best scores: each counted player whose score (+0xCC) is not below group 4's tenth is entered (`menu::enter_best_score_4`); with one entered, the page (`menu::best_overall_scores_loop`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `sub_4A0E60`. The entries are their Rust versions; the page through `part` (1), in
//! the game its Rust version (`screen_loop::run_screen`). Ungated (the page waits for the
//! player). Refuses a player count of 0 or past 4, a missing player, no table, and what an entry refuses, before anything. Self-test: a copy whose page call goes to a stand-in (the entries'
//! to their hooks), against the Rust version, on the game's tables with scores by lot;
//! put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{game_read, game_write, rva};
use crate::menu::screen_loop::{Carried, run_screen};
use crate::menu::standings::*;

const CURRENT_RVA: usize = rva(0x75598C); // dword_75598C
const ENTERED_RVA: usize = rva(0x755920); // dword_755920
// dword_7558F4
// dword_7558F8
// g_yesNoAnswer

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn best_overall_scores(c: &Carried) -> Result<(), Refused> {
    unsafe {
        let players = counted_players()?;
        let table: u32 = game_read(BEST_SCORES_RVA);
        if table == 0 {
            return Err(BadInput("no best-score table").into());
        }
        let mut g = *c;
        game_write(ENTERED_RVA, 0u32);
        for (n, &p) in players.iter().enumerate() {
            game_write(CURRENT_RVA, n as u32);
            g.arg = get(p + 0xCC);
            g.palette = 4 * 0x258 + 0x21C;
            g.cars = table + g.palette;
            // Converted: `rc2k_core::menu::bests::makes_score`.
            if rc2k_core::menu::bests::makes_score(g.arg, get(g.cars + 0x38)) {
                enter_best(Some(4), 0xCC)?;
                game_write(ENTERED_RVA, 1u32);
            }
        }
        g.measure_x = players.len() as u32;
        if game_read::<u32>(ENTERED_RVA) == 1 {
            run_screen(&crate::menu::best_overall_scores_loop::SCREEN, &g)?;
        }
    }
    Ok(())
}
