//! `menu::best_class_scores` (0x4A8608): each class's (0 .. `event.classes`) new best scores: each counted player of the class whose score (+0xB0) is not below the class's tenth is entered (`menu::enter_best_score`); with one entered, `event.class` the class and the page (`menu::best_class_scores_loop`), stopping when the leaving is answered yes (`g_yesNoAnswer` 1).
//!
//! Caller: the event drivers. Refuses a player count of 0 or
//! past 4, a missing player, no classes or more than five, no table, and what an entry refuses,
//! before anything.

use crate::errors::{BadInput, Refused};
use crate::menu::screen_loop::{Entry, run_screen};
use crate::menu::standings::*;

pub fn best_class_scores(menus: &crate::menu::owner::Menus, entry: &Entry) -> Result<(), Refused> {
    let players = counted_players(menus)?;
    let classes: u32 = menus.screens.event.classes.get();
    let (true, true) = (
        classes != 0 && classes <= 5,
        crate::saves::best_times::SCORES.is_taken(),
    ) else {
        return Err(BadInput("no classes, more than five, or no best-score table").into());
    };
    for class in 0..classes {
        menus.screens.event.any_time_qualified.set(false);
        for (n, &p) in players.iter().enumerate() {
            menus.screens.event.current_player.set(n as u32);
            let player_score = p.get().stage_points[6];
            if p.get().class == class {
                // The group's tenth entry's offset (the original leaves its address in
                // esi, which the screen's lists set before anything reads it).
                // The logic is in `rc2k_core::menu::bests::makes_score`.
                let tenth = tenth_best_score(class)?;
                if rc2k_core::menu::bests::makes_score(player_score, tenth) {
                    enter_best(menus, None, |p| p.stage_points[6])?;
                    menus.screens.event.any_time_qualified.set(true);
                }
            }
        }
        if menus.screens.event.any_time_qualified.is_set() {
            menus.screens.event.class.set(class);
            run_screen(&crate::menu::best_class_scores_loop::SCREEN, entry)?;
            if menus.tables.pages.yes_no_answer.get() == 1 {
                break;
            }
        }
    }
    Ok(())
}
