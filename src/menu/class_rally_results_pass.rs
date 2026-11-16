//! `menu::class_rally_results_pass`: standings table A sorted by total time
//! (`menu::sort_standings_by_total`), then for each class from 0 up to `CLASSES`: when
//! a
//! player (1 .. 3: player 4 is not looked for, a 1999 slip kept as is) is in the class's
//! entries of standings table A, the class is numbered (`number_standings`, its count into
//! `event.standings_count`) and its page shown (`menu::show_class_rally_results`), stopping when
//! leaving is answered yes. `loop_flags.left_with_carry` is the class's flag.
//!
//! Refuses a table without its end entry and a class count the loop would run past (at or below the
//! first, or past 0x100), before anything; what the steps refuse.

use crate::errors::Refused;
use crate::menu::owner::{Play, Screens};
use crate::menu::pages_store::MenuPages;
use crate::menu::screen_loop::Entry;
use crate::menu::standings::*;
use crate::menu::standings_steps::number_standings;
use rc2k_core::menu::class_pages;

/// The first class.
fn first_class() -> u32 {
    class_pages::FIRST_RALLY_CLASS
}

pub fn class_rally_results_pass(
    screens: &Screens,
    play: &Play,
    pages: &MenuPages,
    entry: &Entry,
    tables: &crate::saves::best_times::BestTables,
) -> Result<(), Refused> {
    crate::menu::sort_standings_by_total::sort_standings_by_total(screens, play, tables)?;
    let first = first_class();
    let classes = class_pages::classes(first, crate::menu::event_store::CLASSES)?;
    let all: Vec<(u32, u32)> = walked_entries_a(tables)?
        .iter()
        .map(|e| (e.head.group, e.head.player))
        .collect();
    for class in classes {
        screens.loop_flags.left_with_carry.set(false);
        let has_player = class_pages::has_player(all.len(), |n| all[n].0, |n| all[n].1, class);
        if !has_player {
            continue;
        }
        screens.loop_flags.left_with_carry.set(true);
        let numbered = number_standings(play, Which::A, class, tables)?;
        screens.event.standings_count.set(numbered.last_number);
        crate::menu::show_class_rally_results::show_class_rally_results(
            &play.champ_view,
            &screens.event,
            class,
            entry,
            tables,
        )?;
        if pages.yes_no_answer.get() == 1 {
            break;
        }
    }
    Ok(())
}
