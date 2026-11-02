//! `menu::best_overall_scores` (0x4A91BF): the overall new best scores: each counted player whose score (+0xCC) is not below group 4's tenth is entered (`menu::enter_best_score_4`); with one entered, the page (`menu::best_overall_scores_loop`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `sub_4A0E60`. The entries are their Rust versions. Refuses a player count of 0 or past
//! 4, a missing player, no table, and what an entry refuses, before anything.

use crate::errors::Refused;
use crate::menu::event_store::EVENT_STATE;
use crate::menu::screen_loop::{Carried, run_screen};
use crate::menu::standings::*;

// dword_7558F4
// dword_7558F8
// g_yesNoAnswer

/// The Rust version (see the header): the registers it leaves.
pub fn best_overall_scores(c: &Carried) -> Result<(), Refused> {
    let players = counted_players()?;
    let mut g = *c;
    EVENT_STATE.any_time_qualified.set(0u32);
    for (n, &p) in players.iter().enumerate() {
        EVENT_STATE.current_player.set(n as u32);
        g.arg = p.get().word(0xCC);
        g.palette = 4 * 0x258 + 0x21C;
        // The tenth entry's offset (the original leaves its address in esi, which the
        // screen's lists set before anything reads it).
        // Converted: `rc2k_core::menu::bests::makes_score`.
        let score = tenth_best_score(4)?;
        if rc2k_core::menu::bests::makes_score(g.arg, score) {
            enter_best(Some(4), 0xCC)?;
            EVENT_STATE.any_time_qualified.set(1u32);
        }
    }
    g.measure_x = players.len() as u32;
    if EVENT_STATE.any_time_qualified.get() == 1 {
        run_screen(&crate::menu::best_overall_scores_loop::SCREEN, &g)?;
    }
    Ok(())
}
