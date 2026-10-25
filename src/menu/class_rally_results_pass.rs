//! `menu::class_rally_results_pass` (0x4A32A5): standings table A sorted by total time (`menu::sort_standings_by_total`), then for each class from 0 up to `dword_7558F8`:
//! when a player (1 .. 3: the original does not look for player 4, section 16) is in the
//! class's entries of standings table A, the class numbered (`menu::number_standings_a`, its
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
use crate::menu::event_store::EVENT_STATE;
use crate::menu::loop_store::LOOP_FLAGS;
use crate::menu::pages_store::MENU_PAGES;
use crate::menu::screen_loop::Carried;
use crate::menu::standings::*;
use rc2k_core::menu::class_pages;

// dword_75AB40
// g_eventMode

/// The first class (see the header).
fn first_class() -> u32 {
    class_pages::FIRST_RALLY_CLASS
}

/// The Rust version (see the header). `c` the carried values (registers the pages' screens are run with).
pub unsafe fn class_rally_results_pass(c: &Carried) -> Result<(), Refused> {
    unsafe {
        crate::menu::sort_standings_by_total::sort_standings_by_total()?;
        let first = first_class();
        // Converted: `rc2k_core::menu::class_pages` (`classes`, `has_player`).
        let classes = class_pages::classes(first, EVENT_STATE.classes.get())?;
        let table: u32 = TABLE_A_RVA.get();
        let all = entries(table, A_ENTRY)?;
        for class in classes {
            LOOP_FLAGS.ended_b.set(0u32);
            if !class_pages::has_player(
                all.len(),
                |n| get(all[n] + 0x1C),
                |n| get(all[n] + 4),
                class,
            ) {
                continue;
            }
            LOOP_FLAGS.ended_b.set(1u32);
            let numbered = crate::menu::number_standings_a::number_standings_a(class)?;
            let mut g = Carried {
                palette: class,
                ..*c
            };
            numbered.carry_on(&mut g);
            EVENT_STATE.count_a.set(numbered.last_number);
            crate::menu::show_class_rally_results::show_class_rally_results(class, &g)?;
            if MENU_PAGES.yes_no_answer.get() == 1 {
                break;
            }
        }
    }
    Ok(())
}
