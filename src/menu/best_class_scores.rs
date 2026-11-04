//! `menu::best_class_scores` (0x4A8608): each class's (0 .. `dword_7558F8`) new best scores: each counted player of the class whose score (+0xB0) is not below the class's tenth is entered (`menu::enter_best_score`); with one entered, `dword_7558F4` the class and the page (`menu::best_class_scores_loop`), stopping when the leaving is answered yes (`g_yesNoAnswer` 1).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: the event drivers. The entries are their Rust versions. Refuses a player count of 0 or
//! past 4, a missing player, no classes or more than five, no table, and what an entry refuses,
//! before anything.

use crate::errors::{BadInput, Refused};
use crate::menu::event_store::EVENT_STATE;
use crate::menu::pages_store::MENU_PAGES;
use crate::menu::screen_loop::{Entry, run_screen};
use crate::menu::standings::*;

/// The Rust version (see the header): the registers it leaves.
pub fn best_class_scores() -> Result<(), Refused> {
    let players = counted_players()?;
    let classes: u32 = EVENT_STATE.classes.get();
    let (true, true) = (
        classes != 0 && classes <= 5,
        crate::saves::best_times::SCORES.is_taken(),
    ) else {
        return Err(BadInput("no classes, more than five, or no best-score table").into());
    };
    for class in 0..classes {
        EVENT_STATE.any_time_qualified.set(0u32);
        for (n, &p) in players.iter().enumerate() {
            EVENT_STATE.current_player.set(n as u32);
            let player_score = p.get().stage_points[6];
            if p.get().class == class {
                // The group's tenth entry's offset (the original leaves its address in
                // esi, which the screen's lists set before anything reads it).
                // Converted: `rc2k_core::menu::bests::makes_score`.
                let tenth = tenth_best_score(class)?;
                if rc2k_core::menu::bests::makes_score(player_score, tenth) {
                    enter_best(None, |p| p.stage_points[6])?;
                    EVENT_STATE.any_time_qualified.set(1u32);
                }
            }
        }
        if EVENT_STATE.any_time_qualified.get() == 1 {
            EVENT_STATE.class.set(class);
            run_screen(
                &crate::menu::best_class_scores_loop::SCREEN,
                &Entry::default(),
            )?;
            if MENU_PAGES.yes_no_answer.get() == 1 {
                break;
            }
        }
    }
    Ok(())
}
