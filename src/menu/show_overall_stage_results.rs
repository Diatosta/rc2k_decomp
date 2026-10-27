//! `menu::show_overall_stage_results` (0x4A1FE5): the overall stage results: the first player's entry of any group in standings
//! table A (`menu::first_player_a`), its figure held to 0xAB into `dword_7556B4`;
//! `menu::first_page_stage`, `menu::stage_range_caption`; then the page
//! (`menu::overall_stage_results_loop`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `sub_4A0E60`. The steps are their Rust versions. Refuses what the steps refuse, before
//! anything; raises the search's fatal error as the original.

use crate::errors::Refused;
use crate::menu::champ_store::CHAMP_VIEW;
use crate::menu::screen_loop::{Carried, run_screen};
use crate::menu::standings::*;
use rc2k_core::menu::results_pages::{Page, held_figure};

// dword_7558F4
// g_eventMode

/// The Rust version (see the header). `c` the carried values (the registers the page's
/// screen is run with).
pub unsafe fn show_overall_stage_results(c: &Carried) -> Result<(), Refused> {
    unsafe {
        let mut c = Carried {
            palette: ANY_GROUP,
            ..*c
        };
        let found = crate::menu::first_player_a::first_player_a(ANY_GROUP)?;
        found.carry_on(&mut c);
        // Converted: `rc2k_core::menu::results_pages::held_figure`.
        let count = held_figure(Page::OverallStage, found.figure);
        CHAMP_VIEW.count.set(count);
        crate::menu::first_page_stage::carry_on(
            crate::menu::first_page_stage::first_page_stage(),
            &mut c,
        );
        crate::menu::stage_range_caption::stage_range_caption()?;
        run_screen(&crate::menu::overall_stage_results_loop::SCREEN, &c)?;
    }
    Ok(())
}
