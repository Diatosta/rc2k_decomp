//! `menu::class_stage_results_pass` (0x4A1EE6): standings table A sorted by stage time (`menu::sort_standings_by_time`), then for each class from 1 (with `state.menus.class_mode` 2) or 0 up to `event.classes`:
//! when a player (1 .. 3: the original does not look for player 4, section 16) is in the
//! class's entries of standings table A, the class numbered (`number_standings`, its
//! count into `event.count_a`) and its page shown (`menu::show_class_stage_results`), stopping when the
//! leaving is answered yes (`g_yesNoAnswer` 1). `loop_flags.left_with_carry` the class's flag.
//!
//! Callers: the event drivers. Refuses a table without its end
//! entry and a class count the loop would run past (at or below the first, or past 0x100), before
//! anything; what the steps refuse.

use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::menu::owner::Screens;
use crate::menu::screen_loop::Entry;
use crate::menu::standings::*;
use crate::menu::standings_steps::number_standings;
use rc2k_core::menu::class_pages;

// g_eventMode

/// The first class.
fn first_class(screens: &Screens) -> u32 {
    class_pages::first_stage_class(screens.state.menus.class_mode.get())
}

pub fn class_stage_results_pass(menus: &Menus, entry: &Entry) -> Result<(), Refused> {
    let screens = &menus.screens;
    crate::menu::sort_standings_by_time::sort_standings_by_time(menus)?;
    let first = first_class(screens);
    // The logic is in `rc2k_core::menu::class_pages` (`classes`, `has_player`).
    let classes = class_pages::classes(first, menus.screens.event.classes.get())?;
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
        menus.screens.event.count_a.set(numbered.last_number);
        crate::menu::show_class_stage_results::show_class_stage_results(menus, class, entry)?;
        if menus.tables.pages.yes_no_answer.get() == 1 {
            break;
        }
    }
    Ok(())
}
