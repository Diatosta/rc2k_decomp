//! `menu::best_overall_scores` (0x4A91BF): the overall new best scores: each counted player whose score (+0xCC) is not below group 4's tenth is entered (`menu::enter_best_score_4`); with one entered, the page (`menu::best_overall_scores_loop`).
//!
//! Refuses a player count of 0 or past
//! 4, a missing player, no table, and what an entry refuses, before anything.

use crate::errors::Refused;
use crate::menu::screen_loop::{Entry, run_screen};
use crate::menu::standings::*;
// `event.classes`
// g_yesNoAnswer
pub fn best_overall_scores(menus: &crate::menu::owner::Menus) -> Result<(), Refused> {
    let screens = &menus.screens;
    let players = counted_players(menus)?;
    screens.event.any_time_qualified.set(false);
    for (n, &p) in players.iter().enumerate() {
        screens.event.current_player.set(n as u32);
        let player_score = p.get().overall_points[6];
        // The tenth entry's offset (the original leaves its address in esi, which the
        // screen's lists set before anything reads it).
        // The logic is in `rc2k_core::menu::bests::makes_score`.
        let tenth = tenth_best_score(4)?;
        if rc2k_core::menu::bests::makes_score(player_score, tenth) {
            enter_best(menus, Some(4), |p| p.overall_points[6])?;
            screens.event.any_time_qualified.set(true);
        }
    }
    if screens.event.any_time_qualified.is_set() {
        run_screen(
            &crate::menu::best_overall_scores_loop::SCREEN,
            &Entry::new(crate::owners::boundary()),
        )?;
    }
    Ok(())
}
