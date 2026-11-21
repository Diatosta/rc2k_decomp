//! `menu::best_overall_scores`: the overall new best scores: each counted player whose score
//! (`overall_points` total) is not below group 4's tenth is entered (`menu::enter_best_score_4`); with one
//! entered, the page (`menu::best_overall_scores_loop`).
//!
//! Refuses a player count of 0 or past 4, a missing player, no table, and what an entry refuses,
//! before anything.

use crate::errors::Refused;
use crate::menu::screen_id::ScreenId;
use crate::menu::screen_loop::{Entry, run_screen};
use crate::menu::standings::*;
pub(super) fn best_overall_scores(
    menus: &crate::menu::owner::Menus,
    entry: &Entry,
    tables: &crate::saves::best_times::BestTables,
) -> Result<(), Refused> {
    let screens = &menus.screens;
    let players = counted_players(&menus.screens, &menus.play)?;
    screens.event.write().trial.any_qualified = false;
    for (n, &p) in players.iter().enumerate() {
        screens.event.write().current_player = n as u32;
        let player_score = p.get().overall_points[6];
        let tenth = tenth_best_score(tables, 4)?;
        if rc2k_core::menu::bests::makes_score(player_score, tenth) {
            enter_best(&menus.screens, &menus.play, tables, Some(4), |p| {
                p.overall_points[6]
            })?;
            screens.event.write().trial.any_qualified = true;
        }
    }
    if screens.event.read().trial.any_qualified {
        run_screen(ScreenId::BestOverallScores, entry)?;
    }
    Ok(())
}
