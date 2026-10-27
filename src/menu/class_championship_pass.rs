//! `menu::class_championship_pass` (0x4A4A1C): for each class from 1 (0 outside `dword_75AB40` 2 in event mode 0x13) up to `dword_7558F8`:
//! when a player (1 .. 3: the original does not look for player 4, section 16) is in the
//! class's entries of standings table B, the class numbered (`menu::number_standings_b`, its
//! count into `dword_75597C`) and its page shown (`menu::show_class_championship_standings`), stopping when the
//! leaving is answered yes (`g_yesNoAnswer` 1). `dword_755565` the class's flag.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: the event drivers, `sub_4A9583`. The steps are their Rust versions. Refuses a table
//! without its end entry and a class count the loop would run past (at or below the first, or past
//! 0x100), before anything; what the steps refuse.

use crate::errors::Refused;
use crate::menu::champ_store::CHAMP_VIEW;
use crate::menu::event_store::EVENT_STATE;
use crate::menu::loop_store::LOOP_FLAGS;
use crate::menu::pages_store::MENU_PAGES;
use crate::menu::screen_loop::Carried;
use crate::menu::standings::*;
use crate::menu::state_store::MENU_STATE;
use rc2k_core::menu::class_pages;

/// The first class (see the header).
fn first_class() -> u32 {
    class_pages::first_championship_class(MENU_STATE.menus.class_mode.get(), || {
        CHAMP_VIEW.event_mode.get()
    })
}

/// The Rust version (see the header). `c` the carried values (registers the pages' screens are run with).
pub unsafe fn class_championship_pass(c: &Carried) -> Result<(), Refused> {
    unsafe {
        let first = first_class();
        // Converted: `rc2k_core::menu::class_pages` (`classes`, `has_player`).
        let classes = class_pages::classes(first, EVENT_STATE.classes.get())?;
        let all: Vec<(u32, u32)> = walked_entries_b()?
            .iter()
            .map(|e| (e.group, e.player))
            .collect();
        for class in classes {
            LOOP_FLAGS.ended_b.set(0u32);
            let has_player = class_pages::has_player(all.len(), |n| all[n].0, |n| all[n].1, class);
            if !has_player {
                continue;
            }
            LOOP_FLAGS.ended_b.set(1u32);
            let numbered = crate::menu::number_standings_b::number_standings_b(class)?;
            let mut g = Carried {
                palette: class,
                ..*c
            };
            numbered.carry_on(&mut g);
            EVENT_STATE.count_a.set(numbered.last_number);
            crate::menu::show_class_championship_standings::show_class_championship_standings(
                class, &g,
            )?;
            if MENU_PAGES.yes_no_answer.get() == 1 {
                break;
            }
        }
    }
    Ok(())
}
