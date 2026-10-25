//! `menu::class_stage_results_pass` (0x4A1EE6): standings table A sorted by stage time (`menu::sort_standings_by_time`), then for each class from 1 (with `dword_75AB40` 2) or 0 up to `dword_7558F8`:
//! when a player (1 .. 3: the original does not look for player 4, section 16) is in the
//! class's entries of standings table A, the class numbered (`menu::number_standings_a`, its
//! count into `dword_75597C`) and its page shown (`menu::show_class_stage_results`), stopping when the
//! leaving is answered yes (`g_yesNoAnswer` 1). `dword_755565` the class's flag.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: the event drivers. The steps are their Rust versions. Refuses a table without its end
//! entry and a class count the loop would run past (at or below the first, or past 0x100), before
//! anything; what the steps refuse.

use crate::errors::Refused;
use crate::framework::{game_read, game_write, rva};
use crate::menu::screen_loop::Carried;
use crate::menu::standings::*;
use crate::menu::state_store::MENU_STATE;
use rc2k_core::menu::class_pages;

const FLAG_RVA: usize = rva(0x755565); // dword_755565
const COUNT_RVA: usize = rva(0x75597C); // dword_75597C
const CLASSES_RVA: usize = rva(0x7558F8); // dword_7558F8
const ANSWER_RVA: usize = rva(0x75524C); // g_yesNoAnswer
// g_eventMode

/// The first class (see the header).
fn first_class() -> u32 {
    class_pages::first_stage_class(MENU_STATE.menus.class_mode.get())
}

/// The Rust version (see the header). `c` the carried values (registers the pages' screens are run with).
pub unsafe fn class_stage_results_pass(c: &Carried) -> Result<(), Refused> {
    unsafe {
        crate::menu::sort_standings_by_time::sort_standings_by_time()?;
        let first = first_class();
        // Converted: `rc2k_core::menu::class_pages` (`classes`, `has_player`).
        let classes = class_pages::classes(first, game_read(CLASSES_RVA))?;
        let table: u32 = game_read(TABLE_A_RVA);
        let all = entries(table, A_ENTRY)?;
        for class in classes {
            game_write(FLAG_RVA, 0u32);
            if !class_pages::has_player(
                all.len(),
                |n| get(all[n] + 0x1C),
                |n| get(all[n] + 4),
                class,
            ) {
                continue;
            }
            game_write(FLAG_RVA, 1u32);
            let numbered = crate::menu::number_standings_a::number_standings_a(class)?;
            let mut g = Carried {
                palette: class,
                ..*c
            };
            numbered.carry_on(&mut g);
            game_write(COUNT_RVA, numbered.last_number);
            crate::menu::show_class_stage_results::show_class_stage_results(class, &g)?;
            if game_read::<u32>(ANSWER_RVA) == 1 {
                break;
            }
        }
    }
    Ok(())
}
