//! `menu::show_class_championship_standings` (0x4A4AD5): the championship standings of a class: `dword_7558F4` the class (ebx); the first player's entry of class ebx in standings
//! table B (`menu::first_player_b`), its figure held to 0x33 into `dword_7556B4`;
//! `menu::first_page_stage`, `menu::stage_range_caption`; then the page
//! (`menu::class_championship_standings_loop`).
//!
//! In:  ebx the class
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `sub_4A4A1C`. The steps are their Rust versions. Refuses what the steps refuse, before
//! anything; raises the search's fatal error as the original.

use crate::errors::Refused;
use crate::menu::champ_store::CHAMP_VIEW;
use crate::menu::draw_work_store::DRAW_WORK;
use crate::menu::event_store::EVENT_STATE;
use crate::menu::screen_loop::{Carried, run_screen};
use crate::menu::standings::*;
use rc2k_core::menu::results_pages::{Page, held_figure};

// g_eventMode

/// The Rust version (see the header). `c` the carried values (the registers the page's
/// screen is run with).
pub unsafe fn show_class_championship_standings(class: u32, c: &Carried) -> Result<(), Refused> {
    unsafe {
        let mut c = Carried {
            palette: class,
            ..*c
        };
        c.arg = DRAW_WORK.seconds.get(); // seconds
        EVENT_STATE.class.set(class);
        let found = crate::menu::first_player_b::first_player_b(TABLE_B_RVA.get(), class)?;
        found.carry_on(&mut c);
        // Converted: `rc2k_core::menu::results_pages::held_figure`.
        let count = held_figure(Page::ClassChampionship, found.figure);
        CHAMP_VIEW.count.set(count);
        crate::menu::first_page_stage::carry_on(
            crate::menu::first_page_stage::first_page_stage(),
            &mut c,
        );
        crate::menu::stage_range_caption::stage_range_caption();
        run_screen(&crate::menu::class_championship_standings_loop::SCREEN, &c)?;
    }
    Ok(())
}
