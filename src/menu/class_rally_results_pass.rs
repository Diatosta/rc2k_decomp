//! `menu::class_rally_results_pass`: standings table A sorted by total time
//! (`menu::sort_standings_by_total`), then for each class from 0 up to `event.shown.classes`: when
//! a
//! player (1 .. 3: player 4 is not looked for, a 1999 slip kept as is) is in the class's
//! entries of standings table A, the class is numbered (`number_standings`, its count into
//! `event.standings_count`) and its page shown (`menu::show_class_rally_results`), stopping when
//! leaving is answered yes. `loop_flags.left_with_carry` is the class's flag.
//!
//! Callers: the event drivers. Refuses a table without its end entry and a class count the
//! loop would run past (at or below the first, or past 0x100), before anything; what the steps
//! refuse.

use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::menu::screen_loop::Entry;
use crate::menu::standings::*;
use crate::menu::standings_steps::number_standings;
use rc2k_core::menu::class_pages;

/// The first class.
fn first_class() -> u32 {
    class_pages::FIRST_RALLY_CLASS
}

pub fn class_rally_results_pass(menus: &Menus, entry: &Entry) -> Result<(), Refused> {
    crate::menu::sort_standings_by_total::sort_standings_by_total(menus)?;
    let first = first_class();
    let classes = class_pages::classes(first, menus.screens.event.shown.classes.get())?;
    let all: Vec<(u32, u32)> = walked_entries_a()?
        .iter()
        .map(|e| (e.head.group, e.head.player))
        .collect();
    for class in classes {
        menus.screens.loop_flags.left_with_carry.set(false);
        let has_player = class_pages::has_player(all.len(), |n| all[n].0, |n| all[n].1, class);
        if !has_player {
            continue;
        }
        menus.screens.loop_flags.left_with_carry.set(true);
        let numbered = number_standings(menus, Which::A, class)?;
        menus
            .screens
            .event
            .standings_count
            .set(numbered.last_number);
        crate::menu::show_class_rally_results::show_class_rally_results(menus, class, entry)?;
        if menus.tables.pages.yes_no_answer.get() == 1 {
            break;
        }
    }
    Ok(())
}
