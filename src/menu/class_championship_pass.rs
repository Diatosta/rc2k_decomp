//! `menu::class_championship_pass`: for each class from 1 (0 outside `state.menus.class_mode` 2
//! in event mode 0x13) up to `event.shown.classes`: when a player (1 .. 3: player 4 is not looked
//! for,
//! a 1999 slip kept as is) is in the class's entries of standings table B, the class is
//! numbered (`number_standings`, its count into `event.standings_count`) and its page shown
//! (`menu::show_class_championship_standings`), stopping when leaving is answered yes.
//! `loop_flags.left_with_carry` is the class's flag.
//!
//! Callers: the event drivers. Refuses a table without its end entry and a class count the
//! loop would run past (at or below the first, or past 0x100), before anything; what the steps
//! refuse.

use crate::errors::Refused;
use crate::menu::champ_store::ChampView;
use crate::menu::owner::Menus;
use crate::menu::screen_loop::Entry;
use crate::menu::standings::*;
use crate::menu::standings_steps::number_standings;
use crate::menu::state_store::MenuState;
use rc2k_core::menu::class_pages;

/// The first class.
fn first_class(state: &MenuState, champ: &ChampView) -> u32 {
    class_pages::first_championship_class(state.menus.class_mode.get(), || champ.event_mode.get())
}

pub fn class_championship_pass(
    menus: &Menus,
    entry: &Entry,
    tables: &crate::saves::best_times::BestTables,
) -> Result<(), Refused> {
    let first = first_class(&menus.screens.state, &menus.play.champ_view);
    let classes = class_pages::classes(first, menus.screens.event.shown.classes.get())?;
    let all: Vec<(u32, u32)> = walked_entries_b(tables)?
        .iter()
        .map(|e| (e.group, e.player))
        .collect();
    for class in classes {
        menus.screens.loop_flags.left_with_carry.set(false);
        let has_player = class_pages::has_player(all.len(), |n| all[n].0, |n| all[n].1, class);
        if !has_player {
            continue;
        }
        menus.screens.loop_flags.left_with_carry.set(true);
        let numbered = number_standings(&menus.play, Which::B, class, tables)?;
        menus
            .screens
            .event
            .standings_count
            .set(numbered.last_number);
        crate::menu::show_class_championship_standings::show_class_championship_standings(
            &menus.play.champ_view,
            &menus.screens.event,
            class,
            entry,
            tables,
        )?;
        if menus.tables.pages.yes_no_answer.get() == 1 {
            break;
        }
    }
    Ok(())
}
