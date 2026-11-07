//! `menu::class_rally_results_pass` (0x4A32A5): standings table A sorted by total time (`menu::sort_standings_by_total`), then for each class from 0 up to `dword_7558F8`:
//! when a player (1 .. 3: the original does not look for player 4, section 16) is in the
//! class's entries of standings table A, the class numbered (`number_standings`, its
//! count into `dword_75597C`) and its page shown (`menu::show_class_rally_results`), stopping when the
//! leaving is answered yes (`g_yesNoAnswer` 1). `dword_755565` the class's flag.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: the event drivers, `sub_4A94EC`. The steps are their Rust versions. Refuses a table
//! without its end entry and a class count the loop would run past (at or below the first, or past
//! 0x100), before anything; what the steps refuse.

use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::menu::standings::*;
use crate::menu::standings_steps::number_standings;
use rc2k_core::menu::class_pages;

// dword_75AB40
// g_eventMode

/// The first class (see the header).
fn first_class() -> u32 {
    class_pages::FIRST_RALLY_CLASS
}

/// The Rust version (see the header).
pub fn class_rally_results_pass(menus: &Menus) -> Result<(), Refused> {
    crate::menu::sort_standings_by_total::sort_standings_by_total(menus)?;
    let first = first_class();
    // Converted: `rc2k_core::menu::class_pages` (`classes`, `has_player`).
    let classes = class_pages::classes(first, menus.screens.event.classes.get())?;
    let all: Vec<(u32, u32)> = walked_entries_a()?
        .iter()
        .map(|e| (e.head.group, e.head.player))
        .collect();
    for class in classes {
        menus.screens.loop_flags.ended_b.set(0u32);
        let has_player = class_pages::has_player(all.len(), |n| all[n].0, |n| all[n].1, class);
        if !has_player {
            continue;
        }
        menus.screens.loop_flags.ended_b.set(1u32);
        let numbered = number_standings(Which::A, class)?;
        menus.screens.event.count_a.set(numbered.last_number);
        crate::menu::show_class_rally_results::show_class_rally_results(menus, class)?;
        if menus.tables.pages.yes_no_answer.get() == 1 {
            break;
        }
    }
    Ok(())
}
