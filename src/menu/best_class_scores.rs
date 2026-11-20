//! `menu::best_class_scores`: each class's (0 .. `CLASSES`) new best scores. Each
//! counted
//! player of the class whose score (`stage_points` total) is not below the class's tenth is entered
//! (`menu::enter_best_score`); with one entered, `event.shown.class` is set to the class and its
//! page
//! shown (`menu::best_class_scores_loop`), stopping when leaving is answered yes.
//!
//! Refuses a player count of 0 or past 4, a missing player, no table,
//! and what an entry refuses, before anything.

use crate::errors::{BadInput, Refused};
use crate::menu::screen_id::ScreenId;
use crate::menu::screen_loop::{Entry, run_screen};
use crate::menu::standings::*;

pub(super) fn best_class_scores(
    menus: &crate::menu::owner::Menus,
    tables: &crate::saves::best_times::BestTables,
    entry: &Entry,
) -> Result<(), Refused> {
    let players = counted_players(&menus.screens, &menus.play)?;
    let classes = crate::menu::event_store::CLASSES;
    if !tables.scores.is_taken() {
        return Err(BadInput("no best-score table").into());
    }
    for class in 0..classes {
        menus.screens.event.trial.any_qualified.set(false);
        for (n, &p) in players.iter().enumerate() {
            menus.screens.event.current_player.set(n as u32);
            let player_score = p.get().stage_points[6];
            if p.get().class == class {
                let tenth = tenth_best_score(tables, class)?;
                if rc2k_core::menu::bests::makes_score(player_score, tenth) {
                    enter_best(&menus.screens, &menus.play, tables, None, |p| {
                        p.stage_points[6]
                    })?;
                    menus.screens.event.trial.any_qualified.set(true);
                }
            }
        }
        if menus.screens.event.trial.any_qualified.is_set() {
            menus.screens.event.shown.class.set(class);
            run_screen(ScreenId::BestClassScores, entry)?;
            if menus.tables.pages.yes_no_answer.get() == 1 {
                break;
            }
        }
    }
    Ok(())
}
